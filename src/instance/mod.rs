use std::net::TcpListener;

#[allow(clippy::module_name_repetitions)]
pub mod master_instance;
#[allow(clippy::module_name_repetitions)]
pub mod replica_instance;

pub mod client_handler;

/// Trait for running a Redis instance.
pub trait Run {
    type Error;
    /// Runs the Redis instance.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the instance runs successfully, otherwise returns an `Error`.
    ///
    /// # Errors
    ///
    /// If the instance fails to run, an `Error` is returned.
    fn run(&self);
}

/// Trait for listening to incoming connections.
pub trait Listen {
    type Error;
    /// Listens to incoming connections and returns a `TcpListener`.
    ///
    /// # Returns
    ///
    /// Returns a `TcpListener` if the listening is successful, otherwise returns an `Error`.
    ///
    /// # Errors
    ///
    /// If the listener fails to bind to the address, an `Error` is returned.
    fn listen(&self) -> TcpListener;
}
