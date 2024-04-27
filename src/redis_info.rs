use std::fmt::Display;

use crate::{resp::Bulk, ServerConfig};

pub struct RedisInfo {
    role: Role,
}
impl RedisInfo {
    pub fn new(server_config: ServerConfig) -> Self {
        let role = match ServerConfig::replica_of(&server_config) {
            Some(_) => Role::Slave,
            None => Role::Master,
        };
        Self { role }
    }
    pub fn to_bulk_string(&self) -> Bulk {
        Bulk::from_string(format!("{}", self.role).as_str())
    }
}
enum Role {
    Master,
    Slave,
}
impl Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::Master => write!(f, "role:master"),
            Role::Slave => write!(f, "role:slave"),
        }
    }
}
