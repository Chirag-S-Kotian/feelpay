use crate::error::FeelPayError;
use redis::Commands;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Job types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobType {
    ExportJob,
    ReportJob,
    NotificationJob,
    WebhookTestJob,
}

impl JobType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::ExportJob => "export_job",
            Self::ReportJob => "report_job",
            Self::NotificationJob => "notification_job",
            Self::WebhookTestJob => "webhook_test_job",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "export_job" => Some(Self::ExportJob),
            "report_job" => Some(Self::ReportJob),
            "notification_job" => Some(Self::NotificationJob),
            "webhook_test_job" => Some(Self::WebhookTestJob),
            _ => None,
        }
    }
}

/// Job status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobStatus {
    Queued,
    Processing,
    Completed,
    Failed,
}

impl JobStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Queued => "queued",
            Self::Processing => "processing",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "queued" => Some(Self::Queued),
            "processing" => Some(Self::Processing),
            "completed" => Some(Self::Completed),
            "failed" => Some(Self::Failed),
            _ => None,
        }
    }
}

/// Job data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: Uuid,
    pub job_type: String,
    pub payload: serde_json::Value,
    pub status: String,
    pub retry_count: i32,
    pub max_retries: i32,
}

/// JobQueue handles async job processing
pub struct JobQueue {
    redis_client: redis::Client,
}

impl JobQueue {
    /// Create a new JobQueue instance
    pub fn new(redis_client: redis::Client) -> Self {
        Self { redis_client }
    }

    /// Enqueue a job
    pub async fn enqueue(
        &self,
        job_type: JobType,
        payload: serde_json::Value,
        max_retries: i32,
    ) -> Result<Uuid, FeelPayError> {
        let job_id = Uuid::new_v4();
        let job = Job {
            id: job_id,
            job_type: job_type.as_str().to_string(),
            payload,
            status: JobStatus::Queued.as_str().to_string(),
            retry_count: 0,
            max_retries,
        };

        let job_json = serde_json::to_string(&job)
            .map_err(|e| FeelPayError::Internal(format!("Failed to serialize job: {}", e)))?;

        let mut conn = self
            .redis_client
            .get_connection()
            .map_err(|e| FeelPayError::Internal(format!("Redis connection failed: {}", e)))?;

        // Push to queue
        conn.lpush::<_, _, ()>("job_queue", &job_json)
            .map_err(|e| FeelPayError::Internal(format!("Failed to enqueue job: {}", e)))?;

        // Store job metadata
        conn.set_ex::<_, _, ()>(
            format!("job:{}", job_id),
            &job_json,
            3600 * 24, // 24 hours TTL
        )
        .map_err(|e| FeelPayError::Internal(format!("Failed to store job metadata: {}", e)))?;

        Ok(job_id)
    }

    /// Dequeue a job with blocking pop
    pub async fn dequeue(&self, timeout_secs: u64) -> Result<Option<Job>, FeelPayError> {
        let mut conn = self
            .redis_client
            .get_connection()
            .map_err(|e| FeelPayError::Internal(format!("Redis connection failed: {}", e)))?;

        let result: Option<(String, String)> = conn
            .blpop("job_queue", timeout_secs as f64)
            .map_err(|e| FeelPayError::Internal(format!("Failed to dequeue job: {}", e)))?;

        match result {
            Some((_, job_json)) => {
                let job: Job = serde_json::from_str(&job_json).map_err(|e| {
                    FeelPayError::Internal(format!("Failed to deserialize job: {}", e))
                })?;
                Ok(Some(job))
            }
            None => Ok(None),
        }
    }

    /// Mark a job as complete
    pub async fn mark_complete(&self, job_id: Uuid) -> Result<(), FeelPayError> {
        let mut conn = self
            .redis_client
            .get_connection()
            .map_err(|e| FeelPayError::Internal(format!("Redis connection failed: {}", e)))?;

        conn.del::<_, ()>(format!("job:{}", job_id))
            .map_err(|e| FeelPayError::Internal(format!("Failed to delete job: {}", e)))?;

        Ok(())
    }

    /// Retry a job with exponential backoff
    pub async fn retry(&self, job: &Job, backoff_ms: u64) -> Result<(), FeelPayError> {
        if job.retry_count >= job.max_retries {
            // Max retries exceeded, mark as failed
            return self.mark_failed(job.id).await;
        }

        let mut updated_job = job.clone();
        updated_job.retry_count += 1;

        let job_json = serde_json::to_string(&updated_job)
            .map_err(|e| FeelPayError::Internal(format!("Failed to serialize job: {}", e)))?;

        let mut conn = self
            .redis_client
            .get_connection()
            .map_err(|e| FeelPayError::Internal(format!("Redis connection failed: {}", e)))?;

        // Calculate exponential backoff: backoff_ms * 2^retry_count
        let delay_ms = backoff_ms * (2_u64.pow(updated_job.retry_count as u32));
        let delay_secs = (delay_ms / 1000).max(1);

        // Add to delayed queue
        let score = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + delay_secs) as f64;

        conn.zadd::<_, _, _, ()>("job_queue_delayed", &job_json, score)
            .map_err(|e| FeelPayError::Internal(format!("Failed to retry job: {}", e)))?;

        // Update job metadata
        conn.set_ex::<_, _, ()>(format!("job:{}", job.id), &job_json, 3600 * 24)
            .map_err(|e| FeelPayError::Internal(format!("Failed to update job metadata: {}", e)))?;

        Ok(())
    }

    /// Mark a job as failed
    pub async fn mark_failed(&self, job_id: Uuid) -> Result<(), FeelPayError> {
        let mut conn = self
            .redis_client
            .get_connection()
            .map_err(|e| FeelPayError::Internal(format!("Redis connection failed: {}", e)))?;

        conn.del::<_, ()>(format!("job:{}", job_id))
            .map_err(|e| FeelPayError::Internal(format!("Failed to delete job: {}", e)))?;

        Ok(())
    }

    /// Get job status
    pub async fn get_job_status(&self, job_id: Uuid) -> Result<Option<Job>, FeelPayError> {
        let mut conn = self
            .redis_client
            .get_connection()
            .map_err(|e| FeelPayError::Internal(format!("Redis connection failed: {}", e)))?;

        let job_json: Option<String> = conn
            .get(format!("job:{}", job_id))
            .map_err(|e| FeelPayError::Internal(format!("Failed to get job: {}", e)))?;

        match job_json {
            Some(json) => {
                let job: Job = serde_json::from_str(&json).map_err(|e| {
                    FeelPayError::Internal(format!("Failed to deserialize job: {}", e))
                })?;
                Ok(Some(job))
            }
            None => Ok(None),
        }
    }

    /// Process delayed jobs (move ready jobs back to main queue)
    pub async fn process_delayed_jobs(&self) -> Result<(), FeelPayError> {
        let mut conn = self
            .redis_client
            .get_connection()
            .map_err(|e| FeelPayError::Internal(format!("Redis connection failed: {}", e)))?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as f64;

        // Get all jobs with score <= now
        let ready_jobs: Vec<String> = conn
            .zrangebyscore("job_queue_delayed", 0.0, now)
            .map_err(|e| FeelPayError::Internal(format!("Failed to get delayed jobs: {}", e)))?;

        for job_json in ready_jobs {
            // Move to main queue
            conn.lpush::<_, _, ()>("job_queue", &job_json)
                .map_err(|e| FeelPayError::Internal(format!("Failed to move job: {}", e)))?;

            // Remove from delayed queue
            conn.zrem::<_, _, ()>("job_queue_delayed", &job_json)
                .map_err(|e| {
                    FeelPayError::Internal(format!("Failed to remove from delayed: {}", e))
                })?;
        }

        Ok(())
    }

    /// Get queue stats
    pub async fn get_queue_stats(&self) -> Result<QueueStats, FeelPayError> {
        let mut conn = self
            .redis_client
            .get_connection()
            .map_err(|e| FeelPayError::Internal(format!("Redis connection failed: {}", e)))?;

        let queued_count: i64 = conn
            .llen("job_queue")
            .map_err(|e| FeelPayError::Internal(format!("Failed to get queue length: {}", e)))?;

        let delayed_count: i64 = conn.zcard("job_queue_delayed").map_err(|e| {
            FeelPayError::Internal(format!("Failed to get delayed queue length: {}", e))
        })?;

        Ok(QueueStats {
            queued_jobs: queued_count,
            delayed_jobs: delayed_count,
        })
    }
}

/// Queue statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStats {
    pub queued_jobs: i64,
    pub delayed_jobs: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_type_as_str() {
        assert_eq!(JobType::ExportJob.as_str(), "export_job");
        assert_eq!(JobType::ReportJob.as_str(), "report_job");
    }

    #[test]
    fn test_job_status_as_str() {
        assert_eq!(JobStatus::Queued.as_str(), "queued");
        assert_eq!(JobStatus::Processing.as_str(), "processing");
    }
}
