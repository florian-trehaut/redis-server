use std::fmt::Display;

use crate::resp::Bulk;

pub struct RedisInfo {
    role: Role,
}
impl RedisInfo {
    pub fn new() -> Self {
        Self { role: Role::Master }
    }
    pub fn to_bulk_string(&self) -> Bulk {
        Bulk::from_string(format!("role:{}", self.role).as_str())
    }
}
enum Role {
    Master,
}
impl Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::Master => write!(f, "role:master"),
        }
    }
}
