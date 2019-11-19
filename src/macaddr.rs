use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fmt::{self, Display};
use std::ops::{Add, Sub};

const MAC_MAX: u64 = 0xffffffffffffu64;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct MacAddr {
    bytes: [u8; 6],
}

impl MacAddr {
    pub fn new(b1: u8, b2: u8, b3: u8, b4: u8, b5: u8, b6: u8) -> Self {
        MacAddr {
            bytes: [b1, b2, b3, b4, b5, b6],
        }
    }
}

impl Display for MacAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.bytes[0],
            self.bytes[1],
            self.bytes[2],
            self.bytes[3],
            self.bytes[4],
            self.bytes[5]
        )
    }
}

impl Ord for MacAddr {
    fn cmp(&self, other: &Self) -> Ordering {
        for i in 0..6 {
            let result = self.bytes[i].cmp(&other.bytes[i]);
            if result != Ordering::Equal {
                return result;
            }
        }
        Ordering::Equal
    }
}

impl PartialOrd for MacAddr {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl TryFrom<&str> for MacAddr {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if let Ok(n) = u64::from_str_radix(value, 10) {
            if n <= MAC_MAX {
                let bytes = u64_to_bytes(n);
                return Ok(MacAddr { bytes });
            }
        }
        if value.len() != 17 {
            return Err(());
        }
        let mut v = vec![];
        for chunk in value.split(":") {
            if chunk.len() != 2 {
                return Err(());
            }
            if let Ok(n) = u8::from_str_radix(chunk, 16) {
                v.push(n);
            } else {
                return Err(());
            }
        }
        if v.len() != 6 {
            return Err(());
        }
        Ok(MacAddr::new(v[0], v[1], v[2], v[3], v[4], v[5]))
    }
}

impl<N> Add<N> for MacAddr
where
    N: Into<u64>,
{
    type Output = Self;

    fn add(self, rhs: N) -> Self {
        let n = bytes_to_u64(&self.bytes);
        let bytes = u64_to_bytes(n + rhs.into() & MAC_MAX);
        MacAddr { bytes }
    }
}

impl Sub<MacAddr> for MacAddr {
    type Output = i64;

    fn sub(self, rhs: MacAddr) -> Self::Output {
        let n1 = bytes_to_u64(&self.bytes);
        let n2 = bytes_to_u64(&rhs.bytes);
        if n1 > n2 {
            (n1 - n2) as i64
        } else {
            -((n2 - n1) as i64)
        }
    }
}

fn bytes_to_u64(bytes: &[u8; 6]) -> u64 {
    let mut n: u64 = 0;
    for b in bytes {
        n = (n << 8u64) + *b as u64;
    }
    n
}

fn u64_to_bytes(n: u64) -> [u8; 6] {
    let mut bytes: [u8; 6] = [0; 6];
    for i in 0..6 {
        bytes[i] = (n >> ((5 - i) * 8) & 0xff) as u8;
    }
    bytes
}

#[cfg(test)]
mod tests {
    use super::MacAddr;
    use crate::macaddr::MAC_MAX;
    use std::convert::TryFrom;

    #[test]
    fn mac_addr_display() {
        let mac1 = MacAddr::new(0, 1, 2, 3, 4, 5);
        assert_eq!(format!("{}", mac1), "00:01:02:03:04:05");

        let mac2 = MacAddr::new(0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff);
        assert_eq!(format!("{}", mac2), "aa:bb:cc:dd:ee:ff");
    }

    #[test]
    fn mac_addr_ordering() {
        let mac1 = MacAddr::new(0, 1, 2, 3, 4, 5);
        let mac2 = MacAddr::new(0, 1, 2, 3, 4, 6);
        assert!(mac1 < mac2);
        assert!(mac2 > mac1);

        assert!(MacAddr::new(1, 0, 0, 0, 0, 0) > MacAddr::new(0, 0xff, 0xff, 0xff, 0xff, 0xff));
        assert!(MacAddr::new(0, 1, 0, 0, 0, 0) > MacAddr::new(0, 0, 0xff, 0xff, 0xff, 0xff));
        assert!(MacAddr::new(0, 0, 1, 0, 0, 0) > MacAddr::new(0, 0, 0, 0xff, 0xff, 0xff));
        assert!(MacAddr::new(0, 0, 0, 1, 0, 0) > MacAddr::new(0, 0, 0, 0, 0xff, 0xff));
        assert!(MacAddr::new(0, 0, 0, 0, 1, 0) > MacAddr::new(0, 0, 0, 0, 0, 0xff));
        assert!(MacAddr::new(0, 0, 0, 0, 0, 1) > MacAddr::new(0, 0, 0, 0, 0, 0));
    }

    #[test]
    fn mac_addr_add() {
        let mac1 = MacAddr::new(0, 1, 2, 3, 4, 5);
        assert_eq!(format!("{}", mac1 + 1u8), "00:01:02:03:04:06");

        let mac2 = MacAddr::new(9, 10, 11, 12, 13, 14);
        assert_eq!(format!("{}", mac2 + 0x010101010101u64), "0a:0b:0c:0d:0e:0f");

        let mac3 = MacAddr::new(0xff, 0xff, 0xff, 0xff, 0xff, 0xff);
        assert_eq!(format!("{}", mac3 + 1u8), "00:00:00:00:00:00"); // overflow
    }

    #[test]
    fn mac_addr_sub() {
        let mac0 = MacAddr::new(0, 0, 0, 0, 0, 0);
        let mac1 = MacAddr::new(1, 1, 1, 1, 1, 1);
        let mac1num = 1103823438081;
        let mac2 = MacAddr::new(0xff, 0xff, 0xff, 0xff, 0xff, 0xff);

        assert_eq!(mac1 - mac1, 0);
        assert_eq!(mac1 - mac0, mac1num);
        assert_eq!(mac0 - mac1, -mac1num);
        assert_eq!(mac2 - mac0, MAC_MAX as i64);
        assert_eq!(mac0 - mac2, -(MAC_MAX as i64));
    }

    #[test]
    fn mac_addr_try_from() {
        assert_eq!(
            MacAddr::try_from("00:11:22:33:44:55"),
            Ok(MacAddr::new(0, 0x11, 0x22, 0x33, 0x44, 0x55))
        );

        assert_eq!(
            MacAddr::try_from("aa:bb:cc:dd:ee:ff"),
            Ok(MacAddr::new(0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff))
        );
    }

    #[test]
    fn mac_addr_try_from_number() {
        assert_eq!(MacAddr::try_from("0"), Ok(MacAddr::new(0, 0, 0, 0, 0, 0)));

        assert_eq!(
            MacAddr::try_from("100000"),
            Ok(MacAddr::new(0, 0, 0, 0x01, 0x86, 0xa0))
        );

        assert_eq!(
            MacAddr::try_from("281474976710655"), // 0xffffffffffff
            Ok(MacAddr::new(0xff, 0xff, 0xff, 0xff, 0xff, 0xff))
        );
    }

    #[test]
    fn mac_addr_try_from_err() {
        assert_eq!(MacAddr::try_from("00:11:22:33:44:5"), Err(()));

        assert_eq!(MacAddr::try_from("aa:bb:cc:dd:ee:0ff"), Err(()));

        assert_eq!(MacAddr::try_from("aa:bb:cc:dd:ee:fg"), Err(()));

        assert_eq!(
            MacAddr::try_from("281474976710656"), // 0xffffffffffff + 1
            Err(())
        );

        assert_eq!(MacAddr::try_from("aabbccddeeff"), Err(()));
    }
}
