use std::ops::AddAssign;
use std::ops::{Add, Sub};
use std::str::FromStr;

use crate::addr::Addr;
use crate::bundled_iter::{IterBundle, ResettableIterator};
use crate::ipv4addr::IPv4Addr;
use crate::ipv6addr::IPv6Addr;
use crate::macaddr::MacAddr;

#[derive(Debug, PartialEq)]
pub struct AddrRange<T> {
    start: T,
    end: T,
    overflow: bool,
}

impl<T> AddrRange<T> {
    fn new(start: T, end: T) -> Self {
        AddrRange {
            start,
            end,
            overflow: false,
        }
    }
    fn is_ascending(&self) -> bool
    where
        T: PartialOrd,
    {
        if self.overflow {
            self.start >= self.end
        } else {
            self.start <= self.end
        }
    }

    fn within(&self, value: T) -> bool
    where
        T: PartialOrd,
    {
        if self.overflow {
            if self.start <= self.end {
                value <= self.start || value >= self.end
            } else {
                value >= self.start || value <= self.end
            }
        } else if self.start <= self.end {
            value >= self.start && value <= self.end
        } else {
            value <= self.start && value >= self.end
        }
    }

    pub fn into_range<S>(self) -> AddrRange<S>
    where
        T: Into<S>,
    {
        AddrRange {
            start: self.start.into(),
            end: self.end.into(),
            overflow: self.overflow,
        }
    }
}

impl<T> FromStr for AddrRange<T>
where
    T: Copy + FromStr + Rangeable,
{
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if let Some(i) = value.find("+") {
            if i < value.len() {
                let start = T::from_str(&value[0..i]).map_err(|_| ())?;
                let negative = &value[i + 1..i + 2] == "-";
                let end = if negative {
                    start - <T as Rangeable>::Int::from_str(&value[i + 2..]).map_err(|_| ())?
                } else {
                    start + <T as Rangeable>::Int::from_str(&value[i + 1..]).map_err(|_| ())?
                };
                return Ok(AddrRange {
                    start,
                    end,
                    overflow: if negative { start < end } else { start > end },
                });
            }
        } else if let Some(i) = value.find("-") {
            if i < value.len() {
                let start = T::from_str(&value[0..i]).map_err(|_| ())?;
                let end = T::from_str(&value[i + 1..]).map_err(|_| ())?;
                return Ok(AddrRange::new(start, end));
            }
        } else {
            let start = T::from_str(value).map_err(|_| ())?;
            return Ok(AddrRange::new(start, start));
        }
        Err(())
    }
}

pub trait Rangeable:
    Copy
    + PartialOrd
    + Add<<Self as Rangeable>::Int, Output = Self>
    + Sub<<Self as Rangeable>::Int, Output = Self>
{
    type Int: Copy + Into<u128> + From<u32> + AddAssign + FromStr;
}

impl Rangeable for MacAddr {
    type Int = u64;
}

impl Rangeable for IPv4Addr {
    type Int = u32;
}

impl Rangeable for IPv6Addr {
    type Int = u128;
}

impl Rangeable for Addr {
    type Int = u64;
}

impl Rangeable for u128 {
    type Int = u128;
}

pub struct AddrRangeIter<T>
where
    T: Rangeable,
{
    range: AddrRange<T>,
    offset: T::Int,
}

impl<T> IntoIterator for AddrRange<T>
where
    T: Rangeable,
{
    type Item = T;
    type IntoIter = AddrRangeIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        AddrRangeIter {
            range: self,
            offset: 0.into(),
        }
    }
}

impl<T> Iterator for AddrRangeIter<T>
where
    T: Rangeable,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let mut result = None;
        let n = if self.range.is_ascending() {
            self.range.start + self.offset
        } else {
            self.range.start - self.offset
        };
        if self.range.within(n) {
            result = Some(n);
        }
        self.offset += 1.into();
        return result;
    }
}

impl<T> ResettableIterator for AddrRangeIter<T>
where
    T: Rangeable,
{
    fn reset(&mut self) {
        self.offset = 0.into();
    }
}

pub type AddrRanges<T> = IterBundle<AddrRangeIter<T>>;

#[cfg(test)]
mod tests {
    use super::{AddrRange, AddrRanges};
    use crate::ipv4addr::IPv4Addr;
    use crate::macaddr::MacAddr;
    use std::str::FromStr;

    #[test]
    fn addr_range_from_str_with_2macs() {
        assert_eq!(
            AddrRange::<MacAddr>::from_str("00:00:00:00:00:00-00:00:00:00:00:10"),
            Ok(AddrRange::new(
                MacAddr::new(0, 0, 0, 0, 0, 0),
                MacAddr::new(0, 0, 0, 0, 0, 0x10)
            ))
        );

        assert_eq!(
            AddrRange::<MacAddr>::from_str("10-20"),
            Ok(AddrRange::new(
                MacAddr::new(0, 0, 0, 0, 0, 0x0a),
                MacAddr::new(0, 0, 0, 0, 0, 0x14)
            ))
        );

        assert_eq!(
            AddrRange::<MacAddr>::from_str("100-11:22:33:44:55:66"),
            Ok(AddrRange::new(
                MacAddr::new(0, 0, 0, 0, 0, 0x64),
                MacAddr::new(0x11, 0x22, 0x33, 0x44, 0x55, 0x66)
            ))
        );
    }

    #[test]
    fn addr_range_from_str_with_mac_only() {
        assert_eq!(
            AddrRange::<MacAddr>::from_str("aa:bb:cc:dd:ee:ff"),
            Ok(AddrRange::new(
                MacAddr::new(0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff),
                MacAddr::new(0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff),
            ))
        );

        assert_eq!(
            AddrRange::<MacAddr>::from_str("16"),
            Ok(AddrRange::new(
                MacAddr::new(0, 0, 0, 0, 0, 0x010),
                MacAddr::new(0, 0, 0, 0, 0, 0x010),
            ))
        );
    }

    #[test]
    fn addr_range_from_str_err() {
        assert_eq!(
            AddrRange::<MacAddr>::from_str("00:11:22:33:44:55-"),
            Err(())
        );
        assert_eq!(AddrRange::<MacAddr>::from_str("0-1-2"), Err(()));
    }

    #[test]
    fn addr_range_from_str_with_ipv4() {
        assert_eq!(
            AddrRange::<IPv4Addr>::from_str("192.168.0.1-192.168.0.10"),
            Ok(AddrRange::new(
                IPv4Addr::new(192, 168, 0, 1),
                IPv4Addr::new(192, 168, 0, 10)
            ))
        );
    }

    #[test]
    fn addr_range_from_str_with_plus() {
        assert_eq!(
            AddrRange::<IPv4Addr>::from_str("192.168.0.1+10"),
            Ok(AddrRange::new(
                IPv4Addr::new(192, 168, 0, 1),
                IPv4Addr::new(192, 168, 0, 11)
            ))
        );
    }

    #[test]
    fn addr_range_from_str_with_plus_descending() {
        assert_eq!(
            AddrRange::<IPv4Addr>::from_str("192.168.0.10+-9"),
            Ok(AddrRange::new(
                IPv4Addr::new(192, 168, 0, 10),
                IPv4Addr::new(192, 168, 0, 1)
            ))
        );
    }

    #[test]
    fn addr_range_from_str_with_plus_err() {
        assert_eq!(
            AddrRange::<IPv4Addr>::from_str("192.168.0.1+192.168.0.10"),
            Err(())
        );
    }

    #[test]
    fn addr_range_iter_ascending() {
        let range = AddrRange::<MacAddr>::from_str("10-12").unwrap();
        let mut iter = range.into_iter();
        assert_eq!(iter.next(), Some(MacAddr::new(0, 0, 0, 0, 0, 10)));
        assert_eq!(iter.next(), Some(MacAddr::new(0, 0, 0, 0, 0, 11)));
        assert_eq!(iter.next(), Some(MacAddr::new(0, 0, 0, 0, 0, 12)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn addr_range_iter_descending() {
        let range = AddrRange::<MacAddr>::from_str("12-10").unwrap();
        let mut iter = range.into_iter();
        assert_eq!(iter.next(), Some(MacAddr::new(0, 0, 0, 0, 0, 12)));
        assert_eq!(iter.next(), Some(MacAddr::new(0, 0, 0, 0, 0, 11)));
        assert_eq!(iter.next(), Some(MacAddr::new(0, 0, 0, 0, 0, 10)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn addr_range_iter_overflow() {
        let range = AddrRange::<IPv4Addr>::from_str("255.255.255.254+3").unwrap();
        let mut iter = range.into_iter();
        assert_eq!(iter.next(), Some(IPv4Addr::new(255, 255, 255, 254)));
        assert_eq!(iter.next(), Some(IPv4Addr::new(255, 255, 255, 255)));
        assert_eq!(iter.next(), Some(IPv4Addr::new(0, 0, 0, 0)));
        assert_eq!(iter.next(), Some(IPv4Addr::new(0, 0, 0, 1)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn addr_range_iter_underflow() {
        let range = AddrRange::<IPv4Addr>::from_str("0.0.0.1+-3").unwrap();
        let mut iter = range.into_iter();
        assert_eq!(iter.next(), Some(IPv4Addr::new(0, 0, 0, 1)));
        assert_eq!(iter.next(), Some(IPv4Addr::new(0, 0, 0, 0)));
        assert_eq!(iter.next(), Some(IPv4Addr::new(255, 255, 255, 255)));
        assert_eq!(iter.next(), Some(IPv4Addr::new(255, 255, 255, 254)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn addr_range_ranges_iter_one_element() {
        let range = AddrRange::<IPv4Addr>::from_str("1-3").unwrap();
        let mut ranges = AddrRanges::<IPv4Addr>::new();
        ranges.push(range.into_range());
        let mut ranges_iter = ranges.into_iter();
        assert_eq!(ranges_iter.next(), Some(vec![IPv4Addr::new(0, 0, 0, 1)]));
        assert_eq!(ranges_iter.next(), Some(vec![IPv4Addr::new(0, 0, 0, 2)]));
        assert_eq!(ranges_iter.next(), Some(vec![IPv4Addr::new(0, 0, 0, 3)]));
        assert_eq!(ranges_iter.next(), None);
    }

    #[test]
    fn addr_range_ranges_iter_3_elements() {
        let mut ranges = AddrRanges::<IPv4Addr>::new();
        ranges.push(AddrRange::<IPv4Addr>::from_str("1-3").unwrap());
        ranges.push(AddrRange::<IPv4Addr>::from_str("2-6").unwrap());
        ranges.push(AddrRange::<IPv4Addr>::from_str("7-7").unwrap());
        let mut ranges_iter = ranges.into_iter();
        assert_eq!(
            ranges_iter.next(),
            Some(vec![
                IPv4Addr::new(0, 0, 0, 1),
                IPv4Addr::new(0, 0, 0, 2),
                IPv4Addr::new(0, 0, 0, 7),
            ])
        );
        assert_eq!(
            ranges_iter.next(),
            Some(vec![
                IPv4Addr::new(0, 0, 0, 2),
                IPv4Addr::new(0, 0, 0, 3),
                IPv4Addr::new(0, 0, 0, 7),
            ])
        );
        assert_eq!(
            ranges_iter.next(),
            Some(vec![
                IPv4Addr::new(0, 0, 0, 3),
                IPv4Addr::new(0, 0, 0, 4),
                IPv4Addr::new(0, 0, 0, 7),
            ])
        );
        assert_eq!(
            ranges_iter.next(),
            Some(vec![
                IPv4Addr::new(0, 0, 0, 1),
                IPv4Addr::new(0, 0, 0, 5),
                IPv4Addr::new(0, 0, 0, 7),
            ])
        );
        assert_eq!(
            ranges_iter.next(),
            Some(vec![
                IPv4Addr::new(0, 0, 0, 2),
                IPv4Addr::new(0, 0, 0, 6),
                IPv4Addr::new(0, 0, 0, 7),
            ])
        );
        assert_eq!(ranges_iter.next(), None);
    }
}
