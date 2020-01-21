use std::cmp::Ordering;
use std::fmt;
use std::fmt::Display;
use std::ops::{Add, Sub};

use crate::ipv4addr::IPv4Addr;
use crate::ipv6addr::IPv6Addr;
use crate::macaddr::MacAddr;

#[derive(PartialEq, Copy, Clone)]
pub enum Addr {
    IPv4(IPv4Addr),
    IPv6(IPv6Addr),
    Mac(MacAddr),
}

impl Display for Addr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Addr::IPv4(value) => write!(f, "{}", value),
            Addr::IPv6(value) => write!(f, "{}", value),
            Addr::Mac(value) => write!(f, "{}", value),
        }
    }
}

impl From<IPv4Addr> for Addr {
    fn from(value: IPv4Addr) -> Self {
        Addr::IPv4(value)
    }
}

impl From<IPv6Addr> for Addr {
    fn from(value: IPv6Addr) -> Self {
        Addr::IPv6(value)
    }
}

impl From<MacAddr> for Addr {
    fn from(value: MacAddr) -> Self {
        Addr::Mac(value)
    }
}

impl PartialOrd for Addr {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self {
            Addr::IPv4(value) => {
                if let Addr::IPv4(ovalue) = other {
                    return value.partial_cmp(ovalue);
                }
            }
            Addr::IPv6(value) => {
                if let Addr::IPv6(ovalue) = other {
                    return value.partial_cmp(ovalue);
                }
            }
            Addr::Mac(value) => {
                if let Addr::Mac(ovalue) = other {
                    return value.partial_cmp(ovalue);
                }
            }
        }
        None
    }
}

impl<N> Add<N> for Addr
where
    N: Into<u128>,
{
    type Output = Self;

    fn add(self, rhs: N) -> Self::Output {
        match self {
            Addr::IPv4(value) => Addr::IPv4(value + rhs.into() as u32),
            Addr::IPv6(value) => Addr::IPv6(value + rhs),
            Addr::Mac(value) => Addr::Mac(value + rhs.into() as u64),
        }
    }
}

impl<N> Sub<N> for Addr
where
    N: Into<u128>,
{
    type Output = Self;

    fn sub(self, rhs: N) -> Self::Output {
        match self {
            Addr::IPv4(value) => Addr::IPv4(value - rhs.into() as u32),
            Addr::IPv6(value) => Addr::IPv6(value - rhs),
            Addr::Mac(value) => Addr::Mac(value - rhs.into() as u64),
        }
    }
}
