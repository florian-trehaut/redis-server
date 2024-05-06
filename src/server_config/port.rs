use std::{fmt::Display, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[cfg(test)]
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
        assert_eq!(format!("{}", port), "1024");

        let port = Port::new(65535).unwrap();
        assert_eq!(format!("{}", port), "65535");
    }

    #[test]
    fn test_port_error_display() {
        let error = PortError::Reserved;
        assert_eq!(format!("{}", error), "Port number is reserved");

        let error = PortError::NotANumber;
        assert_eq!(format!("{}", error), "Input is not a number");
    }
}
