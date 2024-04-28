use std::{
    fmt::Display,
    net::{Ipv4Addr, ToSocketAddrs},
    str::FromStr,
};

#[derive(Clone)]
pub struct ServerConfig {
    port: Port,
    replica_of: Option<ReplicaOf>,
}
impl ServerConfig {
    pub fn replica_of(&self) -> Option<&ReplicaOf> {
        self.replica_of.as_ref()
    }
    pub fn port(&self) -> &Port {
        &self.port
    }
    pub fn from_args(args: &[String]) -> ServerConfig {
        let mut port = "6379"; // default port
        let mut host_of_replica = None;
        let mut port_of_host = None;

        if let Some(port_arg_position) = args.iter().position(|arg| *arg == "--port") {
            if args.len() > port_arg_position + 1 {
                port = &args[port_arg_position + 1];
            }
        }

        if let Some(replica_arg_position) = args.iter().position(|arg| *arg == "--replicaof") {
            match (
                args.get(replica_arg_position + 1),
                args.get(replica_arg_position + 2),
            ) {
                (Some(host), Some(port)) => {
                    host_of_replica = Some(host.parse::<Host>().unwrap());
                    port_of_host = Some(port.parse::<Port>().unwrap());
                }
                _ => {
                    eprintln!("Invalid arguments for --replicaof");
                    std::process::exit(1);
                }
            }
        }

        let port: Port = port.parse().expect("Invalid port number");
        let replica_of = match (host_of_replica, port_of_host) {
            (Some(host), Some(port)) => Some(ReplicaOf::new(host, port)),
            _ => None,
        };

        ServerConfig { port, replica_of }
    }
}

#[derive(Clone)]
pub struct ReplicaOf {
    host_address: Host,
    port: Port,
}

impl ReplicaOf {
    pub fn new(host: Host, port: Port) -> Self {
        Self {
            host_address: host,
            port,
        }
    }
    pub fn host_address(&self) -> &Host {
        &self.host_address
    }
    pub fn port(&self) -> &Port {
        &self.port
    }
}
#[derive(Clone, Debug)]
pub struct Host(String);
impl Host {
    pub fn get(&self) -> &str {
        &self.0
    }
}
impl ToSocketAddrs for Host {
    type Iter = std::vec::IntoIter<std::net::SocketAddr>;
    fn to_socket_addrs(&self) -> std::io::Result<Self::Iter> {
        match self.0.as_str() {
            "localhost" => "127.0.0.1".to_socket_addrs(),
            _ => self.0.as_str().to_socket_addrs(),
        }
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
