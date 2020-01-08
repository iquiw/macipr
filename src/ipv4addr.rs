use std::cmp::Ordering;
use std::fmt::{self, Display};
use std::net::Ipv4Addr;
use std::ops::{Add, Sub};
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct IPv4Addr(Ipv4Addr);

impl IPv4Addr {
    #[cfg(test)]
    pub fn new(b1: u8, b2: u8, b3: u8, b4: u8) -> Self {
        IPv4Addr(Ipv4Addr::new(b1, b2, b3, b4))
    }
}

impl Display for IPv4Addr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
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

impl FromStr for IPv4Addr {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(n) = u32::from_str_radix(s, 10) {
            return Ok(IPv4Addr::from(n));
        }
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

impl Into<u64> for IPv4Addr {
    fn into(self) -> u64 {
        Into::<u32>::into(self.0) as u64
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
    use std::str::FromStr;

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

    #[test]
    fn ipv4addr_from_str() {
        assert_eq!(IPv4Addr::from_str("0.0.0.0"), Ok(IPv4Addr::new(0, 0, 0, 0)));

        assert_eq!(
            IPv4Addr::from_str("192.168.10.1"),
            Ok(IPv4Addr::new(192, 168, 10, 1))
        );

        assert_eq!(
            IPv4Addr::from_str("255.255.255.255"),
            Ok(IPv4Addr::new(255, 255, 255, 255))
        );
    }

    #[test]
    fn ipv4addr_from_str_number() {
        assert_eq!(IPv4Addr::from_str("0"), Ok(IPv4Addr::new(0, 0, 0, 0)));

        assert_eq!(
            IPv4Addr::from_str("100000"),
            Ok(IPv4Addr::new(0, 0x01, 0x86, 0xa0))
        );

        assert_eq!(
            IPv4Addr::from_str("4294967295"), // 0xffffffff
            Ok(IPv4Addr::new(255, 255, 255, 255))
        );
    }

    #[test]
    fn ipv4addr_from_str_err() {
        assert_eq!(IPv4Addr::from_str("192.168.0."), Err(()));

        assert_eq!(IPv4Addr::from_str("10.0.0.256"), Err(()));

        assert_eq!(IPv4Addr::from_str("172.a.0.1"), Err(()));

        assert_eq!(
            IPv4Addr::from_str("4294967296"), // 0xffffffff + 1
            Err(())
        );

        assert_eq!(IPv4Addr::from_str("192168000001"), Err(()));
    }
}
