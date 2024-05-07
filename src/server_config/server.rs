/// This module contains the implementation of the `ServerConfig` enum and its associated types and functions.
/// The `ServerConfig` enum represents the configuration of a Redis server, which can be either a master or a slave.
/// It provides methods for creating a `ServerConfig` from command line arguments, retrieving the port of the server,
/// and handling errors related to server configuration.
///
/// The `ServerConfig` enum has two variants:
/// - `Master`: Represents the configuration of a master server.
/// - `Slave`: Represents the configuration of a slave server.
///
/// The `MasterConfig` struct represents the configuration of a master server and contains the port of the server.
/// It provides methods for retrieving the port of the server and creating a `MasterConfig` from command line arguments
/// or from a `ServerConfig` enum.
///
/// The `SlaveConfig` struct represents the configuration of a slave server and contains the port of the server and the
/// information about the master server it replicates. It provides methods for retrieving the port and the replica information
/// of the server, creating a `SlaveConfig` from command line arguments or from a `ServerConfig` enum.
///
/// The `ServerConfigError` enum represents the possible errors that can occur during server configuration.
/// It has two variants:
/// - `Master`: Represents an error related to the configuration of a master server.
/// - `Slave`: Represents an error related to the configuration of a slave server.
///
/// The `MasterConfigError` enum represents the possible errors that can occur during the configuration of a master server.
/// It has two variants:
/// - `MissingPort`: Indicates that the port argument is missing.
/// - `InvalidPort`: Indicates that the port argument is invalid.
///
/// The `SlaveConfigError` enum represents the possible errors that can occur during the configuration of a slave server.
/// It has five variants:
/// - `MissingReplicaOf`: Indicates that the replicaof argument is missing.
/// - `MissingReplicaOfHost`: Indicates that the host argument of the replicaof command is missing.
/// - `MissingReplicaOfPort`: Indicates that the port argument of the replicaof command is missing.
/// - `InvalidReplicaOfHost`: Indicates that the host argument of the replicaof command is invalid.
/// - `InvalidReplicaOfPort`: Indicates that the port argument of the replicaof command is invalid.
///
/// The `parse_port` function is a helper function that parses the port argument from the command line arguments.
/// If the port argument is not provided, it defaults to "6379".
use std::fmt::Display;

use crate::{Port, ReplicaOf};

use super::{host::Host, port::Error};

#[derive(Clone, Debug)]
pub enum Config {
    Master(MasterConfig),
    Slave(SlaveConfig),
}
impl Config {
    /// Parses the command line arguments and creates a `ServerConfig` from them.
    ///
    /// # Arguments
    ///
    /// * `args` - The command line arguments.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `ServerConfig` if the arguments are valid, or an error if the arguments are invalid.
    ///
    /// # Errors
    ///
    /// Returns an error if the arguments are invalid or missing.
    pub fn from_args(args: &[&str]) -> Result<Self, ConfigError> {
        if args.iter().any(|arg| *arg == "--replicaof") {
            SlaveConfig::from_args(args)
                .map(Config::Slave)
                .map_err(ConfigError::from)
        } else {
            MasterConfig::from_args(args)
                .map(Config::Master)
                .map_err(ConfigError::from)
        }
    }

    /// Returns the port of the server.
    ///
    /// # Returns
    ///
    /// Returns a reference to the `Port` of the server.
    #[must_use]
    pub const fn port(&self) -> &Port {
        match self {
            Self::Master(config) => config.port(),
            Self::Slave(config) => config.port(),
        }
    }
}

#[derive(Debug)]
pub enum ConfigError {
    Master(MasterConfigError),
    Slave(SlaveConfigError),
}
impl From<MasterConfigError> for ConfigError {
    fn from(err: MasterConfigError) -> Self {
        Self::Master(err)
    }
}
impl From<SlaveConfigError> for ConfigError {
    fn from(err: SlaveConfigError) -> Self {
        Self::Slave(err)
    }
}
impl Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Master(err) => write!(f, "{err}"),
            Self::Slave(err) => write!(f, "{err}"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SlaveConfig {
    port: Port,
    replica_of: ReplicaOf,
}
impl SlaveConfig {
    #[must_use]
    pub const fn replica_of(&self) -> &ReplicaOf {
        &self.replica_of
    }
    #[must_use]
    pub const fn port(&self) -> &Port {
        &self.port
    }
    /// Parses the command line arguments and creates a `SlaveConfig` from them.
    ///
    /// # Arguments
    ///
    /// * `args` - The command line arguments.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `SlaveConfig` if the arguments are valid, or an error if the arguments are invalid.
    ///
    /// # Errors
    ///
    /// Returns an error if the arguments are invalid or missing.
    pub fn from_args(args: &[&str]) -> Result<Self, SlaveConfigError> {
        let Some(replica_arg_position) = args.iter().position(|arg| *arg == "--replicaof") else {
            return Err(SlaveConfigError::MissingReplicaOf);
        };

        let Some(host_of_replica) = args.get(replica_arg_position + 1) else {
            return Err(SlaveConfigError::MissingReplicaOfHost);
        };

        let host_of_replica = host_of_replica
            .parse::<Host>()
            .map_err(|_| SlaveConfigError::InvalidReplicaOfHost)?;

        let Some(port_of_host) = args.get(replica_arg_position + 2) else {
            return Err(SlaveConfigError::MissingReplicaOfPort);
        };
        let port_of_host = port_of_host
            .parse::<Port>()
            .map_err(|_| SlaveConfigError::InvalidReplicaOfPort)?;

        let replica_of = ReplicaOf::new(host_of_replica, port_of_host);

        Ok(Self {
            port: parse_port(args)?,
            replica_of,
        })
    }

    /// Creates a `SlaveConfig` from a `Config` enum.
    ///
    /// # Arguments
    ///
    /// * `config` - The `Config` enum.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `SlaveConfig` if the `Config` is a `Slave`, or an error if the `Config` is not a `Slave`.
    ///
    /// # Errors
    ///
    /// Returns an error if the `Config` is not a `Slave`.
    pub const fn from_server_config(config: Config) -> Result<Self, SlaveConfigError> {
        match config {
            Config::Slave(config) => Ok(config),
            Config::Master(_) => Err(SlaveConfigError::MissingReplicaOf),
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
    InvalidPort(Error),
}
impl From<Error> for SlaveConfigError {
    fn from(err: Error) -> Self {
        Self::InvalidPort(err)
    }
}
impl Display for SlaveConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingReplicaOf => write!(f, "Missing replicaof"),
            Self::MissingReplicaOfHost => write!(f, "Missing replicaof host"),
            Self::MissingReplicaOfPort => write!(f, "Missing replicaof port"),
            Self::InvalidReplicaOfHost => write!(f, "Invalid replicaof host"),
            Self::InvalidReplicaOfPort => write!(f, "Invalid replicaof port"),
            Self::InvalidPort(err) => write!(f, "Invalid port: {err}"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct MasterConfig {
    port: Port,
}
impl MasterConfig {
    #[must_use]
    pub const fn port(&self) -> &Port {
        &self.port
    }
    /// Parses the command line arguments and creates a `MasterConfig` from them.
    ///
    /// # Arguments
    ///
    /// * `args` - The command line arguments.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `MasterConfig` if the arguments are valid, or an error if the arguments are invalid.
    ///
    /// # Errors
    ///
    /// Returns an error if the arguments are invalid or missing.
    pub fn from_args(args: &[&str]) -> Result<Self, MasterConfigError> {
        Ok(Self {
            port: parse_port(args)?,
        })
    }

    /// Creates a `MasterConfig` from a `Config` enum.
    ///
    /// # Arguments
    ///
    /// * `config` - The `Config` enum.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `MasterConfig` if the `Config` is a `Master`, or an error if the `Config` is not a `Master`.
    ///
    /// # Errors
    ///
    /// Returns an error if the `Config` is not a `Master`.
    pub const fn from_server_config(config: Config) -> Result<Self, MasterConfigError> {
        match config {
            Config::Master(config) => Ok(config),
            Config::Slave(_) => Err(MasterConfigError::MissingPort),
        }
    }
}

#[derive(Debug)]
pub enum MasterConfigError {
    MissingPort,
    InvalidPort(Error),
}
impl From<Error> for MasterConfigError {
    fn from(err: Error) -> Self {
        Self::InvalidPort(err)
    }
}
impl Display for MasterConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingPort => write!(f, "Missing port"),
            Self::InvalidPort(err) => write!(f, "Invalid port: {err}"),
        }
    }
}

fn parse_port(args: &[&str]) -> Result<Port, Error> {
    let mut port = "6379"; // default port

    if let Some(port_arg_position) = args.iter().position(|arg| *arg == "--port") {
        if args.len() > port_arg_position + 1 {
            port = args[port_arg_position + 1];
        }
    }

    port.parse()
}
