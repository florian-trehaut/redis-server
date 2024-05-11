//! This module provides the core functionality for a Redis replication client.
//!
//! It includes the following submodules:
//! - `host`: Contains the `Host` struct, which represents a Redis host.
//! - `port`: Contains the `Port` struct, which represents a Redis port.
//! - `replica`: Contains the `Replica` struct, which represents a Redis replica.
//! - `server`: Contains the `Server` struct, which represents a Redis server.
//!
//! It also includes the following types:
//! - `RedisStore`: A thread-safe `HashMap` that stores Redis values.
//! - `ReplicationId`: Represents a replication ID in Redis.
//! - `Offset`: Represents an offset in Redis.

use std::{
    collections::HashMap,
    fmt::Display,
    sync::{Arc, Mutex},
};

use crate::RedisValue;

pub mod host;
pub mod port;
pub mod replica;
pub mod server;

/// A thread-safe `HashMap` that stores Redis values.
pub type RedisStore = Arc<Mutex<HashMap<String, RedisValue>>>;

/// Represents a replication ID in Redis.
///
/// The replication ID is a unique identifier used by Redis for replication.
#[derive(Debug, PartialEq, Eq)]
pub struct ReplicationId(String);
impl ReplicationId {
    /// Returns the length of the replication ID.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Parses a replication ID from an optional string.
    ///
    /// If the string is `None`, a default replication ID of "?" is used.
    pub fn parse(id: Option<String>) -> Self {
        id.map_or_else(|| Self("?".to_string()), Self)
    }
}
impl Display for ReplicationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents an offset in Redis.
///
/// The offset is used in Redis for various operations, such as string manipulation and replication.
#[derive(Debug, PartialEq, Eq)]
pub struct Offset(i8);
impl Offset {
    /// Returns the length of the offset.
    ///
    /// The length is calculated as the number of digits in the offset including sign
    pub fn len(&self) -> usize {
        self.0.to_string().len()
    }

    /// Parses an offset from an optional i8.
    ///
    /// If the i8 is `None`, a default offset of -1 is used.
    pub fn parse(offset: Option<i8>) -> Self {
        offset.map_or(Self(-1), Self)
    }
}
impl Display for Offset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replication_id_len() {
        let id = ReplicationId::parse(Some("12345".to_string()));
        assert_eq!(id.len(), 5);
    }

    #[test]
    fn test_replication_id_parse() {
        let id = ReplicationId::parse(Some("12345".to_string()));
        assert_eq!(id.0, "12345");

        let id = ReplicationId::parse(None);
        assert_eq!(id.0, "?");
    }

    #[test]
    fn test_replication_id_display() {
        let id = ReplicationId::parse(Some("12345".to_string()));
        assert_eq!(format!("{id}"), "12345");
    }

    #[test]
    fn test_offset_len() {
        let offset = Offset::parse(Some(123));
        assert_eq!(offset.len(), 3);
    }

    #[test]
    fn test_offset_parse() {
        let offset = Offset::parse(Some(123));
        assert_eq!(offset.0, 123);

        let offset = Offset::parse(None);
        assert_eq!(offset.0, -1);
    }

    #[test]
    fn test_offset_display() {
        let offset = Offset::parse(Some(123));
        assert_eq!(format!("{offset}"), "123");
    }
}
