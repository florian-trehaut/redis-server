use std::{fmt::Display, net::IpAddr, str::FromStr};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Host(IpAddr);
impl Host {
    #[must_use]
    pub const fn get(&self) -> &IpAddr {
        &self.0
    }
}
impl FromStr for Host {
    type Err = AddrError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "localhost" {
            return Ok(Self(IpAddr::from([127, 0, 0, 1])));
        }
        s.parse::<IpAddr>().map_or_else(
            |_| Err(AddrError::InvalidHost(s.to_string())),
            |ip| Ok(Self(ip)),
        )
    }
}
impl Display for Host {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub enum AddrError {
    InvalidHost(String),
}
impl Display for AddrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidHost(e) => write!(f, "{e} is not a valid host address"),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_host_from_str() {
        let host = "192.168.0.1".parse::<Host>();
        assert!(host.is_ok());
        assert_eq!(
            host.unwrap().get(),
            &"192.168.0.1".parse::<IpAddr>().unwrap()
        );

        let host = "localhost".parse::<Host>();
        assert!(host.is_ok());
        assert_eq!(host.unwrap().get(), &IpAddr::from([127, 0, 0, 1]));

        let host = "invalid_ip".parse::<Host>();
        assert!(host.is_err());
        assert_eq!(
            format!("{}", host.unwrap_err()),
            "invalid_ip is not a valid host address"
        );

        let host = "::1".parse::<Host>();
        assert!(host.is_ok());
        assert_eq!(host.unwrap().get(), &"::1".parse::<IpAddr>().unwrap());
    }

    #[test]
    fn test_host_display() {
        let host = "192.168.0.1".parse::<Host>().unwrap();
        assert_eq!(format!("{host}"), "192.168.0.1");

        let host = "localhost".parse::<Host>().unwrap();
        assert_eq!(format!("{host}"), "127.0.0.1");

        let host = "::1".parse::<Host>().unwrap();
        assert_eq!(format!("{host}"), "::1");
    }

    #[test]
    fn test_host_get() {
        let host = "192.168.0.1".parse::<Host>().unwrap();
        assert_eq!(host.get(), &"192.168.0.1".parse::<IpAddr>().unwrap());

        let host = "localhost".parse::<Host>().unwrap();
        assert_eq!(host.get(), &IpAddr::from([127, 0, 0, 1]));

        let host = "::1".parse::<Host>().unwrap();
        assert_eq!(host.get(), &"::1".parse::<IpAddr>().unwrap());
    }

    #[test]
    fn test_host_addr_error_display() {
        let error = AddrError::InvalidHost("invalid_ip".to_string());
        assert_eq!(format!("{error}"), "invalid_ip is not a valid host address");
    }
}
