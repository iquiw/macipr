use std::convert::TryFrom;
use std::ops::{Add, Range, Sub};

use crate::bundled_iter::{IterBundle, ResettableIterator};

#[derive(Debug, PartialEq)]
pub struct AddrRange<T> {
    pub start: T,
    pub end: T,
}

impl<T> AddrRange<T>
where
    T: Ord,
{
    fn is_ascending(&self) -> bool {
        self.start <= self.end
    }
}

impl<T> From<Range<T>> for AddrRange<T> {
    fn from(range: Range<T>) -> Self {
        AddrRange {
            start: range.start,
            end: range.end,
        }
    }
}

impl<'a, T> TryFrom<&'a str> for AddrRange<T>
where
    T: Copy + TryFrom<&'a str>,
{
    type Error = ();

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        if let Some(i) = value.find("-") {
            if i < value.len() {
                let s1 = &value[0..i];
                let s2 = &value[i + 1..];
                if let Ok((start, end)) =
                    T::try_from(s1).and_then(|mac1| T::try_from(s2).map(|mac2| (mac1, mac2)))
                {
                    return Ok(AddrRange { start, end });
                }
            }
        } else {
            let start = T::try_from(value).map_err(|_| ())?;
            return Ok(AddrRange { start, end: start });
        }
        Err(())
    }
}

pub struct AddrRangeIter<T> {
    range: AddrRange<T>,
    offset: u64,
}

impl<T> IntoIterator for AddrRange<T>
where
    T: Copy + Ord + Add<u64, Output = T> + Sub<u64, Output = T>,
{
    type Item = T;
    type IntoIter = AddrRangeIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        AddrRangeIter {
            range: self,
            offset: 0,
        }
    }
}

impl<T> Iterator for AddrRangeIter<T>
where
    T: Copy + Ord + Add<u64, Output = T> + Sub<u64, Output = T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let mut result = None;
        if self.range.is_ascending() {
            let n = self.range.start + self.offset;
            if n <= self.range.end {
                result = Some(n);
            }
        } else {
            let n = self.range.start - self.offset;
            if n >= self.range.end {
                result = Some(n);
            }
        }
        self.offset += 1;
        return result;
    }
}

impl<T> ResettableIterator for AddrRangeIter<T>
where
    T: Copy + Ord + Add<u64, Output = T> + Sub<u64, Output = T>,
{
    fn reset(&mut self) {
        self.offset = 0;
    }
}

pub type AddrRanges<T> = IterBundle<AddrRangeIter<T>>;

#[cfg(test)]
mod tests {
    use super::{AddrRange, AddrRanges};
    use crate::macaddr::MacAddr;
    use std::convert::TryFrom;

    #[test]
    fn addr_range_try_from_with_2macs() {
        assert_eq!(
            AddrRange::<MacAddr>::try_from("00:00:00:00:00:00-00:00:00:00:00:10"),
            Ok(AddrRange {
                start: MacAddr::new(0, 0, 0, 0, 0, 0),
                end: MacAddr::new(0, 0, 0, 0, 0, 0x10)
            })
        );

        assert_eq!(
            AddrRange::<MacAddr>::try_from("10-20"),
            Ok(AddrRange {
                start: MacAddr::new(0, 0, 0, 0, 0, 0x0a),
                end: MacAddr::new(0, 0, 0, 0, 0, 0x14)
            })
        );

        assert_eq!(
            AddrRange::<MacAddr>::try_from("100-11:22:33:44:55:66"),
            Ok(AddrRange {
                start: MacAddr::new(0, 0, 0, 0, 0, 0x64),
                end: MacAddr::new(0x11, 0x22, 0x33, 0x44, 0x55, 0x66)
            })
        );
    }

    #[test]
    fn addr_range_try_from_with_mac_only() {
        assert_eq!(
            AddrRange::<MacAddr>::try_from("aa:bb:cc:dd:ee:ff"),
            Ok(AddrRange {
                start: MacAddr::new(0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff),
                end: MacAddr::new(0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff),
            })
        );

        assert_eq!(
            AddrRange::<MacAddr>::try_from("16"),
            Ok(AddrRange {
                start: MacAddr::new(0, 0, 0, 0, 0, 0x010),
                end: MacAddr::new(0, 0, 0, 0, 0, 0x010),
            })
        );
    }

    #[test]
    fn addr_range_try_from_err() {
        assert_eq!(
            AddrRange::<MacAddr>::try_from("00:11:22:33:44:55-"),
            Err(())
        );
        assert_eq!(AddrRange::<MacAddr>::try_from("0-1-2"), Err(()));
    }

    #[test]
    fn addr_range_iter_ascending() {
        let range = AddrRange::<MacAddr>::try_from("10-12").unwrap();
        let mut iter = range.into_iter();
        assert_eq!(iter.next(), Some(MacAddr::new(0, 0, 0, 0, 0, 10)));
        assert_eq!(iter.next(), Some(MacAddr::new(0, 0, 0, 0, 0, 11)));
        assert_eq!(iter.next(), Some(MacAddr::new(0, 0, 0, 0, 0, 12)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn addr_range_iter_descending() {
        let range = AddrRange::<MacAddr>::try_from("12-10").unwrap();
        let mut iter = range.into_iter();
        assert_eq!(iter.next(), Some(MacAddr::new(0, 0, 0, 0, 0, 12)));
        assert_eq!(iter.next(), Some(MacAddr::new(0, 0, 0, 0, 0, 11)));
        assert_eq!(iter.next(), Some(MacAddr::new(0, 0, 0, 0, 0, 10)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn addr_range_ranges_iter_one_element() {
        let range = AddrRange::<MacAddr>::try_from("1-3").unwrap();
        let mut ranges = AddrRanges::<MacAddr>::new();
        ranges.push(range);
        let mut ranges_iter = ranges.into_iter();
        assert_eq!(
            ranges_iter.next(),
            Some(vec![MacAddr::new(0, 0, 0, 0, 0, 1)])
        );
        assert_eq!(
            ranges_iter.next(),
            Some(vec![MacAddr::new(0, 0, 0, 0, 0, 2)])
        );
        assert_eq!(
            ranges_iter.next(),
            Some(vec![MacAddr::new(0, 0, 0, 0, 0, 3)])
        );
        assert_eq!(ranges_iter.next(), None);
    }

    #[test]
    fn addr_range_ranges_iter_3_elements() {
        let mut ranges = AddrRanges::<MacAddr>::new();
        ranges.push(AddrRange::<MacAddr>::try_from("1-3").unwrap());
        ranges.push(AddrRange::<MacAddr>::try_from("2-6").unwrap());
        ranges.push(AddrRange::<MacAddr>::try_from("7-7").unwrap());
        let mut ranges_iter = ranges.into_iter();
        assert_eq!(
            ranges_iter.next(),
            Some(vec![
                MacAddr::new(0, 0, 0, 0, 0, 1),
                MacAddr::new(0, 0, 0, 0, 0, 2),
                MacAddr::new(0, 0, 0, 0, 0, 7),
            ])
        );
        assert_eq!(
            ranges_iter.next(),
            Some(vec![
                MacAddr::new(0, 0, 0, 0, 0, 2),
                MacAddr::new(0, 0, 0, 0, 0, 3),
                MacAddr::new(0, 0, 0, 0, 0, 7),
            ])
        );
        assert_eq!(
            ranges_iter.next(),
            Some(vec![
                MacAddr::new(0, 0, 0, 0, 0, 3),
                MacAddr::new(0, 0, 0, 0, 0, 4),
                MacAddr::new(0, 0, 0, 0, 0, 7),
            ])
        );
        assert_eq!(
            ranges_iter.next(),
            Some(vec![
                MacAddr::new(0, 0, 0, 0, 0, 1),
                MacAddr::new(0, 0, 0, 0, 0, 5),
                MacAddr::new(0, 0, 0, 0, 0, 7),
            ])
        );
        assert_eq!(
            ranges_iter.next(),
            Some(vec![
                MacAddr::new(0, 0, 0, 0, 0, 2),
                MacAddr::new(0, 0, 0, 0, 0, 6),
                MacAddr::new(0, 0, 0, 0, 0, 7),
            ])
        );
        assert_eq!(ranges_iter.next(), None);
    }
}
