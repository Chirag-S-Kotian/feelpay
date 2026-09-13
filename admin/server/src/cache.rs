use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};
use std::time::{Duration, Instant};

struct CacheEntry {
    value: Value,
    expires_at: Instant,
}

static CACHE: OnceLock<RwLock<HashMap<String, CacheEntry>>> = OnceLock::new();

fn get_cache() -> &'static RwLock<HashMap<String, CacheEntry>> {
    CACHE.get_or_init(|| RwLock::new(HashMap::new()))
}

pub struct AdminCache;

impl AdminCache {
    /// Retrieve a cached value if present and not expired.
    pub fn get<T: DeserializeOwned>(key: &str) -> Option<T> {
        let now = Instant::now();
        let cache = get_cache().read().ok()?;
        if let Some(entry) = cache.get(key) {
            if entry.expires_at > now {
                return serde_json::from_value(entry.value.clone()).ok();
            }
        }
        None
    }

    /// Store a value in the cache with a specified TTL.
    pub fn set<T: Serialize>(key: &str, value: T, ttl: Duration) {
        if let Ok(val) = serde_json::to_value(value) {
            if let Ok(mut cache) = get_cache().write() {
                cache.insert(
                    key.to_string(),
                    CacheEntry {
                        value: val,
                        expires_at: Instant::now() + ttl,
                    },
                );
            }
        }
    }

    /// Invalidate all entries starting with the given prefix.
    pub fn invalidate_prefix(prefix: &str) {
        if let Ok(mut cache) = get_cache().write() {
            cache.retain(|k, _| !k.starts_with(prefix));
        }
    }

    /// Evict all expired entries or clear entirely.
    pub fn clear() {
        if let Ok(mut cache) = get_cache().write() {
            cache.clear();
        }
    }
}
