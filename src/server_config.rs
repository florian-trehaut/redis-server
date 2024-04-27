use std::{fmt::Display, net::Ipv4Addr, str::FromStr};

#[derive(Clone)]
pub struct ServerConfig {
    replica_of: Option<ReplicaOf>,
}
impl ServerConfig {
    pub fn new(replica_of: Option<ReplicaOf>) -> Self {
        Self { replica_of }
    }

    pub fn replica_of(&self) -> Option<&ReplicaOf> {
        self.replica_of.as_ref()
    }
}

#[derive(Clone)]
pub struct ReplicaOf {
    _host: Host,
    _port: Port,
}

impl ReplicaOf {
    pub fn new(host: Host, port: Port) -> Self {
        Self {
            _host: host,
            _port: port,
        }
    }
}
#[derive(Clone, Debug)]
pub struct Host(String);
impl Host {
    pub fn get(&self) -> &str {
        &self.0
    }
}
impl FromStr for Host {
    type Err = HostAddrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "localhost" => Ok(Self("localhost".to_string())),
            _ => match s.parse::<Ipv4Addr>() {
                Ok(_) => Ok(Self(s.to_string())),
                Err(_) => Err(HostAddrError::InvalidHost),
            },
        }
    }
}
impl Display for Host {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct Port(u16);

impl Port {
    pub fn new(port: u16) -> Result<Self, PortError> {
        match port {
            0..=1023 => Err(PortError::Reserved),
            1024..=65535 => Ok(Self(port)),
        }
    }

    pub fn get(&self) -> u16 {
        self.0
    }
}

impl FromStr for Port {
    type Err = PortError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<u16>() {
            Ok(port) => Self::new(port),
            Err(_) => Err(PortError::NotANumber),
        }
    }
}
impl Display for Port {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
#[derive(Debug)]
pub enum PortError {
    Reserved,
    NotANumber,
}
impl Display for PortError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PortError::Reserved => write!(f, "Port number is reserved"),
            PortError::NotANumber => write!(f, "Input is not a number"),
        }
    }
}
#[derive(Debug)]
pub enum HostAddrError {
    InvalidHost,
}
impl Display for HostAddrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HostAddrError::InvalidHost => write!(f, "Invalid host address"),
        }
    }
}
