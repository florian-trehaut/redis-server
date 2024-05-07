use std::{fmt::Display, str::FromStr};

/// Represents a port number.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Port(u16);

impl Port {
    /// Creates a new `Port` instance.
    ///
    /// # Arguments
    ///
    /// * `port` - The port number.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `Port` instance if the port number is valid, or a `PortError` if the port number is reserved.
    ///
    /// # Errors
    ///
    /// Returns a `PortError::Reserved` if the port number is reserved.
    pub const fn new(port: u16) -> Result<Self, Error> {
        match port {
            0..=1023 => Err(Error::Reserved),
            1024..=65535 => Ok(Self(port)),
        }
    }

    /// Returns the port number.
    ///
    /// # Returns
    ///
    /// Returns the port number as a `u16`.
    #[must_use]
    pub const fn get(&self) -> u16 {
        self.0
    }
}

impl FromStr for Port {
    type Err = Error;

    /// Parses a string into a `Port` instance.
    ///
    /// # Arguments
    ///
    /// * `s` - The string to parse.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `Port` instance if the string can be parsed into a valid port number, or a `PortError` if the string is not a valid number or the port number is reserved.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<u16>().map_or(Err(Error::NotANumber), Self::new)
    }
}

impl Display for Port {
    /// Formats the `Port` instance as a string.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter.
    ///
    /// # Returns
    ///
    /// Returns a `std::fmt::Result` indicating whether the formatting was successful.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents an error that can occur when working with a `Port`.
#[derive(Debug)]
pub enum Error {
    /// The port number is reserved.
    Reserved,
    /// The input is not a number.
    NotANumber,
}

impl Display for Error {
    /// Formats the `PortError` instance as a string.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter.
    ///
    /// # Returns
    ///
    /// Returns a `std::fmt::Result` indicating whether the formatting was successful.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Reserved => write!(f, "Port number is reserved"),
            Self::NotANumber => write!(f, "Input is not a number"),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_port_new() {
        let port = Port::new(1024);
        assert!(port.is_ok());
        assert_eq!(port.unwrap().get(), 1024);

        let port = Port::new(65535);
        assert!(port.is_ok());
        assert_eq!(port.unwrap().get(), 65535);

        let port = Port::new(0);
        assert!(port.is_err());
        assert_eq!(format!("{}", port.unwrap_err()), "Port number is reserved");

        let port = Port::new(1023);
        assert!(port.is_err());
        assert_eq!(format!("{}", port.unwrap_err()), "Port number is reserved");
    }

    #[test]
    fn test_port_from_str() {
        let port = "1024".parse::<Port>();
        assert!(port.is_ok());
        assert_eq!(port.unwrap().get(), 1024);

        let port = "65535".parse::<Port>();
        assert!(port.is_ok());
        assert_eq!(port.unwrap().get(), 65535);

        let port = "0".parse::<Port>();
        assert!(port.is_err());
        assert_eq!(format!("{}", port.unwrap_err()), "Port number is reserved");

        let port = "1023".parse::<Port>();
        assert!(port.is_err());
        assert_eq!(format!("{}", port.unwrap_err()), "Port number is reserved");

        let port = "invalid".parse::<Port>();
        assert!(port.is_err());
        assert_eq!(format!("{}", port.unwrap_err()), "Input is not a number");
    }

    #[test]
    fn test_port_display() {
        let port = Port::new(1024).unwrap();
        assert_eq!(format!("{port}"), "1024");

        let port = Port::new(65535).unwrap();
        assert_eq!(format!("{port}"), "65535");
    }

    #[test]
    fn test_port_error_display() {
        let error = Error::Reserved;
        assert_eq!(format!("{error}"), "Port number is reserved");

        let error = Error::NotANumber;
        assert_eq!(format!("{error}"), "Input is not a number");
    }
}
