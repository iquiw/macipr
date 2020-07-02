use std::fmt::{self, Display};
use std::net::Ipv6Addr;
use std::ops::{Add, Sub};
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq, Copy, Clone, PartialOrd, Ord)]
pub struct IPv6Addr(Ipv6Addr);

impl IPv6Addr {
    #[cfg(test)]
    pub fn new(n1: u16, n2: u16, n3: u16, n4: u16, n5: u16, n6: u16, n7: u16, n8: u16) -> Self {
        IPv6Addr(Ipv6Addr::new(n1, n2, n3, n4, n5, n6, n7, n8))
    }
}

impl Display for IPv6Addr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for IPv6Addr {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(n) = u128::from_str_radix(s, 10) {
            return Ok(IPv6Addr::from(n));
        }
        Ok(IPv6Addr(Ipv6Addr::from_str(s).map_err(|_| ())?))
    }
}

impl From<u128> for IPv6Addr {
    fn from(n: u128) -> Self {
        IPv6Addr(Ipv6Addr::from(n))
    }
}

impl Into<u128> for IPv6Addr {
    fn into(self) -> u128 {
        self.0.into()
    }
}

impl<N> Add<N> for IPv6Addr
where
    N: Into<u128>,
{
    type Output = Self;

    fn add(self, rhs: N) -> Self::Output {
        let n: u128 = self.0.into();
        let (add, _) = n.overflowing_add(rhs.into());
        IPv6Addr(Ipv6Addr::from(add))
    }
}

impl<N> Sub<N> for IPv6Addr
where
    N: Into<u128>,
{
    type Output = Self;

    fn sub(self, rhs: N) -> Self::Output {
        let n: u128 = self.0.into();
        let (sub, _) = n.overflowing_sub(rhs.into());
        IPv6Addr(Ipv6Addr::from(sub))
    }
}

#[cfg(test)]
mod tests {
    use super::IPv6Addr;
    use std::str::FromStr;

    #[test]
    fn ipv6addr_add() {
        assert_eq!(
            IPv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0) + 1u128,
            IPv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)
        );
        assert_eq!(
            IPv6Addr::new(10, 11, 12, 13, 14, 15, 16, 17) + 0x00010001000100010001000100010001u128,
            IPv6Addr::new(11, 12, 13, 14, 15, 16, 17, 18)
        );
        assert_eq!(
            IPv6Addr::new(0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff) + 1u128,
            IPv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0)
        );
    }

    #[test]
    fn ipv6addr_sub() {
        assert_eq!(
            IPv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0) - 1u32,
            IPv6Addr::new(0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff)
        );
        assert_eq!(
            IPv6Addr::new(10, 11, 12, 13, 14, 15, 16, 17) - 0x00010001000100010001000100010001u128,
            IPv6Addr::new(9, 10, 11, 12, 13, 14, 15, 16)
        );
        assert_eq!(
            IPv6Addr::new(0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff) - 1u32,
            IPv6Addr::new(0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xfffe)
        );
    }

    #[test]
    fn ipv6addr_from_str() {
        assert_eq!(
            IPv6Addr::from_str("::"),
            Ok(IPv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0))
        );

        assert_eq!(
            IPv6Addr::from_str("fe80::1"),
            Ok(IPv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 1))
        );

        assert_eq!(
            IPv6Addr::from_str("ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff"),
            Ok(IPv6Addr::new(
                0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff
            ))
        );
    }

    #[test]
    fn ipv6addr_from_str_number() {
        assert_eq!(
            IPv6Addr::from_str("0"),
            Ok(IPv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0))
        );

        assert_eq!(
            IPv6Addr::from_str("100000"),
            Ok(IPv6Addr::new(0, 0, 0, 0, 0, 0, 0x01, 0x86a0))
        );

        assert_eq!(
            IPv6Addr::from_str("340282366920938463463374607431768211455"), // 0xffffffffffffffffffffffffffffffffffff
            Ok(IPv6Addr::new(
                0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff
            ))
        );
    }

    #[test]
    fn ipv6addr_from_str_err() {
        assert_eq!(IPv6Addr::from_str("::0::1"), Err(()));

        assert_eq!(IPv6Addr::from_str("::10000"), Err(()));

        assert_eq!(IPv6Addr::from_str("::fgff"), Err(()));

        assert_eq!(
            IPv6Addr::from_str("340282366920938463463374607431768211456"), // 0xffffffffffffffffffffffffffffffffffff + 1
            Err(())
        );

        assert_eq!(IPv6Addr::from_str("ff801"), Err(()));
    }
}
