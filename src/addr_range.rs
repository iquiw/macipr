use std::convert::TryFrom;
use std::ops::Range;

#[derive(Debug, PartialEq)]
struct AddrRange<T> {
    pub start: T,
    pub end: Option<T>,
}

impl<T> From<Range<T>> for AddrRange<T> {
    fn from(range: Range<T>) -> Self {
        AddrRange {
            start: range.start,
            end: Some(range.end),
        }
    }
}

impl<'a, T> TryFrom<&'a str> for AddrRange<T>
where
    T: TryFrom<&'a str>,
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
                    return Ok(AddrRange {
                        start,
                        end: Some(end),
                    });
                }
            }
        } else {
            let start = T::try_from(value).map_err(|_| ())?;
            return Ok(AddrRange { start, end: None });
        }
        Err(())
    }
}

#[cfg(test)]
mod tests {
    use super::AddrRange;
    use crate::macaddr::MacAddr;
    use std::convert::TryFrom;

    #[test]
    fn addr_range_try_from_with_2macs() {
        assert_eq!(
            AddrRange::<MacAddr>::try_from("00:00:00:00:00:00-00:00:00:00:00:10"),
            Ok(AddrRange {
                start: MacAddr::new(0, 0, 0, 0, 0, 0),
                end: Some(MacAddr::new(0, 0, 0, 0, 0, 0x10))
            })
        );

        assert_eq!(
            AddrRange::<MacAddr>::try_from("10-20"),
            Ok(AddrRange {
                start: MacAddr::new(0, 0, 0, 0, 0, 0x0a),
                end: Some(MacAddr::new(0, 0, 0, 0, 0, 0x14))
            })
        );

        assert_eq!(
            AddrRange::<MacAddr>::try_from("100-11:22:33:44:55:66"),
            Ok(AddrRange {
                start: MacAddr::new(0, 0, 0, 0, 0, 0x64),
                end: Some(MacAddr::new(0x11, 0x22, 0x33, 0x44, 0x55, 0x66))
            })
        );
    }

    #[test]
    fn addr_range_try_from_with_mac_only() {
        assert_eq!(
            AddrRange::<MacAddr>::try_from("aa:bb:cc:dd:ee:ff"),
            Ok(AddrRange {
                start: MacAddr::new(0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff),
                end: None
            })
        );

        assert_eq!(
            AddrRange::<MacAddr>::try_from("16"),
            Ok(AddrRange {
                start: MacAddr::new(0, 0, 0, 0, 0, 0x010),
                end: None
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
}
