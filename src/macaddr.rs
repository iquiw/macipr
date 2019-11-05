use std::fmt::{self, Display};
use std::ops::Add;

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

    #[test]
    fn mac_addr_display() {
        let mac1 = MacAddr::new(0, 1, 2, 3, 4, 5);
        assert_eq!(format!("{}", mac1), "00:01:02:03:04:05");

        let mac2 = MacAddr::new(0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff);
        assert_eq!(format!("{}", mac2), "aa:bb:cc:dd:ee:ff");
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
}
