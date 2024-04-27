use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

#[derive(Clone)]
pub struct RedisValue {
    value: String,
    expiration: Option<Instant>,
}
impl RedisValue {
    pub fn new(value: String, expiration: Option<Duration>) -> RedisValue {
        let expiration = expiration.map(|expiration| Instant::now() + expiration);
        RedisValue { value, expiration }
    }
    pub fn value(&self) -> String {
        self.value.to_string()
    }
    pub fn expiration(&self) -> Option<Instant> {
        self.expiration
    }
}
pub type RedisStore = Arc<Mutex<HashMap<String, RedisValue>>>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_redis_value_new() {
        let value = "test".to_string();
        let expiration = Some(Duration::from_secs(1));
        let redis_value = RedisValue::new(value.clone(), expiration);
        assert_eq!(redis_value.value(), value);
        assert!(redis_value.expiration().is_some());
    }

    #[test]
    fn test_redis_value_no_expiration() {
        let value = "test".to_string();
        let redis_value = RedisValue::new(value.clone(), None);
        assert_eq!(redis_value.value(), value);
        assert!(redis_value.expiration().is_none());
    }

    #[test]
    fn test_redis_value_expiration() {
        let value = "test".to_string();
        let expiration = Some(Duration::from_secs(1));
        let redis_value = RedisValue::new(value.clone(), expiration);
        sleep(Duration::from_secs(2));
        assert!(Instant::now() >= redis_value.expiration().unwrap());
    }

    #[test]
    fn test_redis_store() {
        let store: RedisStore = Arc::new(Mutex::new(HashMap::new()));
        let mut store = store.lock().unwrap();
        let value = RedisValue::new("test".to_string(), None);
        store.insert("key".to_string(), value.clone());
        assert_eq!(store.get("key").unwrap().value(), value.value());
    }
}
