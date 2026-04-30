use crate::error::FeelPayError;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

/// Cursor structure for pagination
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Cursor {
    /// The sort key value (e.g., created_at timestamp or UUID)
    pub sort_key: String,
    /// ID of the last record for stable pagination
    pub last_id: String,
    /// Timestamp when cursor was created (for expiration)
    pub created_at: DateTime<Utc>,
}

impl Cursor {
    /// Encode cursor to base64 string
    pub fn encode(&self) -> Result<String, FeelPayError> {
        let json = serde_json::to_string(self)
            .map_err(|e| FeelPayError::Internal(format!("Cursor serialization failed: {}", e)))?;
        Ok(BASE64.encode(json))
    }

    /// Decode cursor from base64 string
    pub fn decode(encoded: &str) -> Result<Self, FeelPayError> {
        let bytes = BASE64
            .decode(encoded)
            .map_err(|_| FeelPayError::BadRequest("Invalid cursor encoding".into()))?;

        let json = String::from_utf8(bytes)
            .map_err(|_| FeelPayError::BadRequest("Invalid cursor format".into()))?;

        serde_json::from_str(&json)
            .map_err(|_| FeelPayError::BadRequest("Invalid cursor structure".into()))
    }
}

/// Pagination direction
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum PaginationDirection {
    #[default]
    Forward,
    Backward,
}

/// Paginated response wrapper
#[derive(Debug, Clone, Serialize, ToSchema)]
#[schema(example = json!({
    "data": [],
    "next_cursor": "eyJzb3J0X2tleSI6IjIwMjQtMDEtMDFUMDA6MDA6MDBaIiwibGFzdF9pZCI6IjEyMzQ1Njc4LTkwYWJjZGVmIn0=",
    "previous_cursor": null,
    "has_more": true,
    "total_count": 150
}))]
pub struct PaginatedResponse<T> {
    /// The data items for current page
    pub data: Vec<T>,
    /// Cursor for next page
    pub next_cursor: Option<String>,
    /// Cursor for previous page
    pub previous_cursor: Option<String>,
    /// Whether there are more items
    pub has_more: bool,
    /// Total count (optional, for informational purposes)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_count: Option<i64>,
}

impl<T> PaginatedResponse<T> {
    /// Create a new paginated response
    pub fn new(data: Vec<T>, next_cursor: Option<String>, has_more: bool) -> Self {
        Self {
            data,
            next_cursor,
            previous_cursor: None,
            has_more,
            total_count: None,
        }
    }

    /// Create with total count
    pub fn with_total(mut self, total: i64) -> Self {
        self.total_count = Some(total);
        self
    }
}

/// Pagination parameters
#[derive(Debug, Clone, Deserialize, Serialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct PaginationParams {
    /// Cursor for next page (base64 encoded)
    #[param(
        example = "eyJzb3J0X2tleSI6IjIwMjQtMDEtMDFUMDA6MDA6MDBaIiwibGFzdF9pZCI6IjEyMzQ1Njc4LTkwYWJjZGVmIn0="
    )]
    pub cursor: Option<String>,
    /// Number of records per page (default 20, max 100)
    #[param(example = 20, minimum = 1, maximum = 100)]
    pub limit: Option<i64>,
}


impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            cursor: None,
            limit: Some(20),
        }
    }
}

impl PaginationParams {
    /// Validate and normalize pagination params
    pub fn normalize(&self) -> Self {
        Self {
            cursor: self.cursor.clone(),
            limit: Some(self.limit.unwrap_or(20).clamp(1, 100)),
        }
    }

    /// Decode cursor if present
    pub fn decode_cursor(&self) -> Result<Option<Cursor>, FeelPayError> {
        match &self.cursor {
            Some(c) => Ok(Some(Cursor::decode(c)?)),
            None => Ok(None),
        }
    }
}

/// Build cursor from sort key and last ID
pub fn build_cursor(sort_key: &str, last_id: &str) -> String {
    let cursor = Cursor {
        sort_key: sort_key.to_string(),
        last_id: last_id.to_string(),
        created_at: Utc::now(),
    };
    cursor.encode().unwrap_or_default()
}

/// Extract sort key and ID from cursor
pub fn extract_cursor_data(cursor: &str) -> Option<(String, String)> {
    Cursor::decode(cursor).ok().map(|c| (c.sort_key, c.last_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_encode_decode() {
        let cursor = Cursor {
            sort_key: "2024-01-01T00:00:00Z".to_string(),
            last_id: "12345678-90abcdef".to_string(),
            created_at: Utc::now(),
        };

        let encoded = cursor.encode().unwrap();
        let decoded = Cursor::decode(&encoded).unwrap();

        assert_eq!(cursor.sort_key, decoded.sort_key);
        assert_eq!(cursor.last_id, decoded.last_id);
    }

    #[test]
    fn test_pagination_params_normalize() {
        let params = PaginationParams {
            cursor: None,
            limit: Some(150),
        };

        let normalized = params.normalize();
        assert_eq!(normalized.limit, Some(100));

        let params2 = PaginationParams {
            cursor: None,
            limit: Some(-5),
        };

        let normalized2 = params2.normalize();
        assert_eq!(normalized2.limit, Some(1));
    }

    #[test]
    fn test_invalid_cursor() {
        let result = Cursor::decode("invalid-base64!");
        assert!(result.is_err());
    }
}
use sqlx::PgPool;

/// Cursor-based query builder for efficient pagination
pub struct CursorQueryBuilder {
    table: String,
    sort_column: String,
    id_column: String,
    conditions: Vec<String>,
    order_desc: bool,
}

impl CursorQueryBuilder {
    /// Create a new cursor query builder
    pub fn new(table: &str, sort_column: &str, id_column: &str) -> Self {
        Self {
            table: table.to_string(),
            sort_column: sort_column.to_string(),
            id_column: id_column.to_string(),
            conditions: vec!["1=1".to_string()],
            order_desc: true,
        }
    }

    /// Add a condition to the query
    pub fn with_condition(mut self, condition: &str) -> Self {
        self.conditions.push(condition.to_string());
        self
    }

    /// Set sort direction
    pub fn order_descending(mut self, desc: bool) -> Self {
        self.order_desc = desc;
        self
    }

    /// Build the SQL query for cursor-based pagination
    pub fn build_query(&self, cursor: Option<&Cursor>, limit: i64) -> String {
        let order = if self.order_desc { "DESC" } else { "ASC" };
        let comp = if self.order_desc { "<" } else { ">" };
        let comp_eq = if self.order_desc { "<=" } else { ">=" };

        let where_clause = if let Some(c) = cursor {
            format!(
                "WHERE ({}.{} {} '{}' OR ({}.{} = '{}' AND {}.{} {} '{}')) AND {}",
                self.table,
                self.sort_column,
                comp,
                c.sort_key,
                self.table,
                self.sort_column,
                c.sort_key,
                self.table,
                self.id_column,
                comp_eq,
                c.last_id,
                self.conditions.join(" AND ")
            )
        } else {
            format!("WHERE {}", self.conditions.join(" AND "))
        };

        format!(
            "SELECT * FROM {} {} ORDER BY {} {} LIMIT {}",
            self.table,
            where_clause,
            self.sort_column,
            order,
            limit + 1
        )
    }

    /// Build count query
    pub fn build_count_query(&self) -> String {
        format!(
            "SELECT COUNT(*) as total FROM {} WHERE {}",
            self.table,
            self.conditions.join(" AND ")
        )
    }
}

/// Execute a cursor-based query and return paginated results
pub async fn execute_cursor_query<T, F>(
    pool: &PgPool,
    builder: &CursorQueryBuilder,
    cursor: Option<&Cursor>,
    limit: i64,
    mapper: F,
) -> Result<PaginatedResponse<T>, FeelPayError>
where
    F: FnMut(sqlx::postgres::PgRow) -> Result<T, sqlx::Error>,
{
    let query = builder.build_query(cursor, limit);

    let rows = sqlx::query(&query)
        .fetch_all(pool)
        .await
        .map_err(|e| FeelPayError::Internal(format!("Query failed: {}", e)))?;

    let has_more = rows.len() > limit as usize;
    let data: Vec<T> = rows
        .into_iter()
        .take(limit as usize)
        .map(mapper)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| FeelPayError::Internal(format!("Row mapping failed: {}", e)))?;

    let next_cursor = if has_more {
        data.last().map(|_| {
            // In real implementation, extract sort key from last record
            build_cursor("placeholder", "placeholder")
        })
    } else {
        None
    };

    Ok(PaginatedResponse::new(data, next_cursor, has_more))
}
