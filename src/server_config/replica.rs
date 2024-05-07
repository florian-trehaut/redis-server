use std::fmt::Display;

use crate::Port;

use super::host::Host;

#[derive(Clone, Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct ReplicaOf {
    host_address: Host,
    port: Port,
}

impl ReplicaOf {
    #[must_use]
    pub const fn new(host: Host, port: Port) -> Self {
        Self {
            host_address: host,
            port,
        }
    }
    #[must_use]
    pub const fn host_address(&self) -> &Host {
        &self.host_address
    }
    #[must_use]
    pub const fn port(&self) -> &Port {
        &self.port
    }
}

impl Display for ReplicaOf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.host_address, self.port)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_replica_of_new() {
        let host = "192.168.0.1".parse::<Host>().unwrap();
        let port = Port::new(8080).unwrap();
        let replica_of = ReplicaOf::new(host.clone(), port.clone());

        assert_eq!(replica_of.host_address(), &host);
        assert_eq!(replica_of.port(), &port);
    }

    #[test]
    fn test_replica_of_display() {
        let host = "192.168.0.1".parse::<Host>().unwrap();
        let port = Port::new(8080).unwrap();
        let replica_of = ReplicaOf::new(host, port);

        assert_eq!(format!("{replica_of}"), "192.168.0.1:8080");
    }
}
