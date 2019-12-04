use std::cmp::Ordering;
use std::convert::TryFrom;
use std::net::Ipv4Addr;
use std::ops::{Add, Sub};
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct IPv4Addr(Ipv4Addr);

impl IPv4Addr {
    pub fn new(b1: u8, b2: u8, b3: u8, b4: u8) -> Self {
        IPv4Addr(Ipv4Addr::new(b1, b2, b3, b4))
    }
}

impl Ord for IPv4Addr {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for IPv4Addr {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl TryFrom<&str> for IPv4Addr {
    type Error = ();

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Ok(IPv4Addr(Ipv4Addr::from_str(s).map_err(|_| ())?))
    }
}

impl From<u32> for IPv4Addr {
    fn from(n: u32) -> Self {
        IPv4Addr(Ipv4Addr::from(n))
    }
}

impl Into<u32> for IPv4Addr {
    fn into(self) -> u32 {
        self.0.into()
    }
}

impl<N> Add<N> for IPv4Addr
where
    N: Into<u32>,
{
    type Output = Self;

    fn add(self, rhs: N) -> Self::Output {
        let n: u32 = self.0.into();
        let (add, _) = n.overflowing_add(rhs.into());
        IPv4Addr(Ipv4Addr::from(add))
    }
}

impl<N> Sub<N> for IPv4Addr
where
    N: Into<u32>,
{
    type Output = Self;

    fn sub(self, rhs: N) -> Self::Output {
        let n: u32 = self.0.into();
        let (sub, _) = n.overflowing_sub(rhs.into());
        IPv4Addr(Ipv4Addr::from(sub))
    }
}

#[cfg(test)]
mod tests {
    use super::IPv4Addr;

    #[test]
    fn ipv4addr_add() {
        assert_eq!(IPv4Addr::new(0, 0, 0, 0) + 1u32, IPv4Addr::new(0, 0, 0, 1));
        assert_eq!(
            IPv4Addr::new(10, 11, 12, 13) + 0x01010101u32,
            IPv4Addr::new(11, 12, 13, 14)
        );
        assert_eq!(
            IPv4Addr::new(255, 255, 255, 255) + 1u32,
            IPv4Addr::new(0, 0, 0, 0)
        );
    }

    #[test]
    fn ipv4addr_sub() {
        assert_eq!(
            IPv4Addr::new(0, 0, 0, 0) - 1u32,
            IPv4Addr::new(255, 255, 255, 255)
        );
        assert_eq!(
            IPv4Addr::new(10, 11, 12, 13) - 0x01010101u32,
            IPv4Addr::new(9, 10, 11, 12)
        );
        assert_eq!(
            IPv4Addr::new(255, 255, 255, 255) - 1u32,
            IPv4Addr::new(255, 255, 255, 254)
        );
    }
}
