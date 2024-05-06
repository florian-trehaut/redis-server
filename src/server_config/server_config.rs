use std::fmt::Display;

use crate::{Port, ReplicaOf};

use super::{host::Host, port::PortError};

#[derive(Clone, Debug)]
pub enum ServerConfig {
    Master(MasterConfig),
    Slave(SlaveConfig),
}
impl ServerConfig {
    pub fn from_args(args: &[&str]) -> Result<ServerConfig, ServerConfigError> {
        if args.iter().any(|arg| *arg == "--replicaof") {
            SlaveConfig::from_args(args)
                .map(ServerConfig::Slave)
                .map_err(ServerConfigError::from)
        } else {
            MasterConfig::from_args(args)
                .map(ServerConfig::Master)
                .map_err(ServerConfigError::from)
        }
    }
    pub fn port(&self) -> &Port {
        match self {
            ServerConfig::Master(config) => config.port(),
            ServerConfig::Slave(config) => config.port(),
        }
    }
}

#[derive(Debug)]
pub enum ServerConfigError {
    Master(MasterConfigError),
    Slave(SlaveConfigError),
}
impl From<MasterConfigError> for ServerConfigError {
    fn from(err: MasterConfigError) -> Self {
        ServerConfigError::Master(err)
    }
}
impl From<SlaveConfigError> for ServerConfigError {
    fn from(err: SlaveConfigError) -> Self {
        ServerConfigError::Slave(err)
    }
}
impl Display for ServerConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerConfigError::Master(err) => write!(f, "{}", err),
            ServerConfigError::Slave(err) => write!(f, "{}", err),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SlaveConfig {
    port: Port,
    replica_of: ReplicaOf,
}
impl SlaveConfig {
    pub fn replica_of(&self) -> &ReplicaOf {
        &self.replica_of
    }
    pub fn port(&self) -> &Port {
        &self.port
    }
    pub fn from_args(args: &[&str]) -> Result<SlaveConfig, SlaveConfigError> {
        let replica_arg_position = match args.iter().position(|arg| *arg == "--replicaof") {
            Some(position) => position,
            None => {
                return Err(SlaveConfigError::MissingReplicaOf);
            }
        };

        let host_of_replica = match args.get(replica_arg_position + 1) {
            Some(host) => host,
            None => {
                return Err(SlaveConfigError::MissingReplicaOfHost);
            }
        };
        let host_of_replica = host_of_replica
            .parse::<Host>()
            .map_err(|_| SlaveConfigError::InvalidReplicaOfHost)?;

        let port_of_host = match args.get(replica_arg_position + 2) {
            Some(port) => port,
            None => {
                return Err(SlaveConfigError::MissingReplicaOfPort);
            }
        };
        let port_of_host = port_of_host
            .parse::<Port>()
            .map_err(|_| SlaveConfigError::InvalidReplicaOfPort)?;

        let replica_of = ReplicaOf::new(host_of_replica, port_of_host);

        Ok(SlaveConfig {
            port: parse_port(args)?,
            replica_of,
        })
    }
    pub fn from_server_config(config: ServerConfig) -> Result<SlaveConfig, SlaveConfigError> {
        match config {
            ServerConfig::Slave(config) => Ok(config),
            _ => Err(SlaveConfigError::MissingReplicaOf),
        }
    }
}

#[derive(Debug)]
pub enum SlaveConfigError {
    MissingReplicaOf,
    MissingReplicaOfHost,
    MissingReplicaOfPort,
    InvalidReplicaOfHost,
    InvalidReplicaOfPort,
    InvalidPort(PortError),
}
impl From<PortError> for SlaveConfigError {
    fn from(err: PortError) -> Self {
        SlaveConfigError::InvalidPort(err)
    }
}
impl Display for SlaveConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SlaveConfigError::MissingReplicaOf => write!(f, "Missing replicaof"),
            SlaveConfigError::MissingReplicaOfHost => write!(f, "Missing replicaof host"),
            SlaveConfigError::MissingReplicaOfPort => write!(f, "Missing replicaof port"),
            SlaveConfigError::InvalidReplicaOfHost => write!(f, "Invalid replicaof host"),
            SlaveConfigError::InvalidReplicaOfPort => write!(f, "Invalid replicaof port"),
            SlaveConfigError::InvalidPort(err) => write!(f, "Invalid port: {}", err),
        }
    }
}

#[derive(Clone, Debug)]
pub struct MasterConfig {
    port: Port,
}
impl MasterConfig {
    pub fn port(&self) -> &Port {
        &self.port
    }
    pub fn from_args(args: &[&str]) -> Result<MasterConfig, MasterConfigError> {
        Ok(MasterConfig {
            port: parse_port(args)?,
        })
    }
    pub fn from_server_config(config: ServerConfig) -> Result<MasterConfig, MasterConfigError> {
        match config {
            ServerConfig::Master(config) => Ok(config),
            _ => Err(MasterConfigError::MissingPort),
        }
    }
}

#[derive(Debug)]
pub enum MasterConfigError {
    MissingPort,
    InvalidPort(PortError),
}
impl From<PortError> for MasterConfigError {
    fn from(err: PortError) -> Self {
        MasterConfigError::InvalidPort(err)
    }
}
impl Display for MasterConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MasterConfigError::MissingPort => write!(f, "Missing port"),
            MasterConfigError::InvalidPort(err) => write!(f, "Invalid port: {}", err),
        }
    }
}

fn parse_port(args: &[&str]) -> Result<Port, PortError> {
    let mut port = "6379"; // default port

    if let Some(port_arg_position) = args.iter().position(|arg| *arg == "--port") {
        if args.len() > port_arg_position + 1 {
            port = &args[port_arg_position + 1];
        }
    }

    port.parse()
}
