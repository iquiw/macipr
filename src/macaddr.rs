use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fmt::{self, Display};
use std::ops::Add;

#[derive(Debug, PartialEq, Eq)]
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

    fn add(self, other: N) -> Self {
        let mut n: u64 = 0;
        for b in &self.bytes {
            n = (n << 8u64) + *b as u64;
        }
        let sum: u64 = n + other.into() & 0xffffffffffffu64;
        let mut bytes: [u8; 6] = [0; 6];
        for i in 0..6 {
            bytes[i] = (dbg!(sum >> ((5 - i) * 8) & 0xff)) as u8;
        }
        MacAddr { bytes }
    }
}

#[cfg(test)]
mod tests {
    use super::MacAddr;
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
    fn mac_addr_try_from_err() {
        assert_eq!(MacAddr::try_from("00:11:22:33:44:5"), Err(()));

        assert_eq!(MacAddr::try_from("aa:bb:cc:dd:ee:0ff"), Err(()));

        assert_eq!(MacAddr::try_from("aa:bb:cc:dd:ee:fg"), Err(()));
    }
}
