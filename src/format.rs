use std::error::Error;
use std::fmt::{self, Display};
use std::io::Write;
use std::str::FromStr;

use crate::addr::Addr;
use crate::addr_range::{AddrRange, AddrRanges};
use crate::ipv4addr::IPv4Addr;
use crate::ipv6addr::{IPv6Addr, IPv6FullAddr};
use crate::macaddr::MacAddr;

#[derive(Debug, PartialEq)]
pub enum Format {
    IPv4Addr,
    IPv6Addr,
    IPv6FullAddr,
    MacAddr,
    Number,
    RawString(String),
}

impl Format {
    fn is_arg_required(&self) -> bool {
        match self {
            Format::RawString(_) => false,
            _ => true,
        }
    }
}

impl Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Format::IPv4Addr => write!(f, "IPv4 address"),
            Format::IPv6Addr => write!(f, "IPv6 address"),
            Format::IPv6FullAddr => write!(f, "IPv6 full address"),
            Format::MacAddr => write!(f, "MAC address"),
            Format::Number => write!(f, "Number"),
            _ => write!(f, "Raw string"),
        }
    }
}

#[derive(Debug, PartialEq)]
enum FormatState {
    Normal,
    Percent,
    Escape,
}

#[derive(Debug, PartialEq)]
pub struct FormatError {
    msg: String,
}

impl Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl Error for FormatError {}

pub fn format_macipr<W>(
    writer: &mut W,
    fmt_str: &str,
    args: &Vec<String>,
) -> Result<(), FormatError>
where
    W: Write,
{
    let mut ranges = AddrRanges::<Addr>::new();
    let mut offset = 0;
    let fmts = parse_format(fmt_str)?;
    for fmt in &fmts {
        if fmt.is_arg_required() {
            if let Some(s) = args.get(offset) {
                let range = if *fmt == Format::IPv4Addr {
                    AddrRange::<IPv4Addr>::from_str(s.as_ref()).map(|r| r.into_range())
                } else if *fmt == Format::IPv6Addr || *fmt == Format::IPv6FullAddr {
                    AddrRange::<IPv6Addr>::from_str(s.as_ref()).map(|r| r.into_range())
                } else if *fmt == Format::MacAddr {
                    AddrRange::<MacAddr>::from_str(s.as_ref()).map(|r| r.into_range())
                } else {
                    AddrRange::<u128>::from_str(s.as_ref()).map(|r| r.into_range())
                }
                .map_err(|_| FormatError {
                    msg: format!("Invalid {}", fmt),
                })?;
                ranges.push(range);
            } else {
                return Err(FormatError {
                    msg: "Insufficient number of arguments".to_string(),
                });
            }
            offset += 1;
        }
    }
    if offset != args.len() {
        return Err(FormatError {
            msg: "Unexpected argument".to_string(),
        });
    }
    if offset == 0 {
        if let Format::RawString(s) = &fmts[0] {
            return Ok(write!(writer, "{}\n", s).map_err(|e| FormatError {
                msg: format!("{}", e),
            })?);
        }
    }
    for v in ranges {
        let mut iter = v.iter();
        for fmt in &fmts {
            match fmt {
                Format::RawString(s) => write!(writer, "{}", s),
                Format::IPv6FullAddr => {
                    if let Addr::IPv6(value) = iter.next().unwrap() {
                        write!(writer, "{}", IPv6FullAddr::wrap(*value))
                    } else {
                        return Err(FormatError { msg: "IPv6 expected".to_string() })
                    }
                }
                _ => write!(writer, "{}", iter.next().unwrap()),
            }
            .map_err(|e| FormatError {
                msg: format!("{}", e),
            })?;
        }
        write!(writer, "\n").map_err(|e| FormatError {
            msg: format!("{}", e),
        })?;
    }
    Ok(())
}

fn parse_format(fmt_str: &str) -> Result<Vec<Format>, FormatError> {
    let mut fmts = vec![];
    let mut buf = String::new();
    let mut state = FormatState::Normal;
    for c in fmt_str.chars() {
        if state == FormatState::Percent {
            state = FormatState::Normal;
            if c == '%' {
                buf.push('%');
            } else {
                if !buf.is_empty() {
                    fmts.push(Format::RawString(buf));
                    buf = String::new();
                }
                match c {
                    'i' => fmts.push(Format::IPv4Addr),
                    'x' => fmts.push(Format::IPv6Addr),
                    'X' => fmts.push(Format::IPv6FullAddr),
                    'm' => fmts.push(Format::MacAddr),
                    'n' => fmts.push(Format::Number),
                    _ => {
                        return Err(FormatError {
                            msg: "Unexpected character after %".to_string(),
                        });
                    }
                }
            }
        } else if state == FormatState::Escape {
            state = FormatState::Normal;
            match c {
                '\\' => buf.push('\\'),
                'n' => buf.push('\n'),
                _ => {
                    return Err(FormatError {
                        msg: "Unexpected character after \\".to_string(),
                    })
                }
            }
        } else {
            if c == '%' {
                state = FormatState::Percent;
                continue;
            } else if c == '\\' {
                state = FormatState::Escape;
                continue;
            }
            buf.push(c);
        }
    }
    if !buf.is_empty() {
        fmts.push(Format::RawString(buf));
    }
    Ok(fmts)
}

#[cfg(test)]
mod tests {
    use super::format_macipr;
    use super::{parse_format, Format, FormatError};

    #[test]
    fn parse_format_empty() {
        assert_eq!(parse_format(""), Ok(vec![]));
    }

    #[test]
    fn parse_format_raw_string() {
        assert_eq!(
            parse_format("foo bar"),
            Ok(vec![Format::RawString("foo bar".to_string())])
        );
    }

    #[test]
    fn parse_format_percent_escape() {
        assert_eq!(
            parse_format("foo %% bar"),
            Ok(vec![Format::RawString("foo % bar".to_string())])
        );
    }

    #[test]
    fn parse_format_macaddr() {
        assert_eq!(parse_format("%m"), Ok(vec![Format::MacAddr]));
    }

    #[test]
    fn parse_format_ipv4addr() {
        assert_eq!(parse_format("%i"), Ok(vec![Format::IPv4Addr]));
    }

    #[test]
    fn parse_format_ipv6addr() {
        assert_eq!(parse_format("%x"), Ok(vec![Format::IPv6Addr]));
    }

    #[test]
    fn parse_format_ipv6fulladdr() {
        assert_eq!(parse_format("%X"), Ok(vec![Format::IPv6FullAddr]));
    }

    #[test]
    fn parse_format_number() {
        assert_eq!(parse_format("%n"), Ok(vec![Format::Number]));
    }

    #[test]
    fn parse_format_escape() {
        assert_eq!(
            parse_format("\\n"),
            Ok(vec![Format::RawString("\n".to_string())])
        );
        assert_eq!(
            parse_format("\\\\"),
            Ok(vec![Format::RawString("\\".to_string())])
        );
        assert_eq!(
            parse_format("\\\\%m\\n%i\\\\foo"),
            Ok(vec![
                Format::RawString("\\".to_string()),
                Format::MacAddr,
                Format::RawString("\n".to_string()),
                Format::IPv4Addr,
                Format::RawString("\\foo".to_string()),
            ])
        );
    }

    #[test]
    fn parse_format_error() {
        assert_eq!(
            parse_format("%k"),
            Err(FormatError {
                msg: "Unexpected character after %".to_string()
            })
        );
        assert_eq!(
            parse_format("\\r"),
            Err(FormatError {
                msg: "Unexpected character after \\".to_string()
            })
        );
    }

    fn fmt_macipr_str(fmt_str: &str, args: &Vec<String>) -> Result<String, FormatError> {
        let mut v = vec![];
        format_macipr(&mut v, fmt_str, args)?;
        Ok(String::from_utf8_lossy(&v).to_string())
    }

    #[test]
    fn format_macaddr_empty_arg() {
        let args = vec![];
        assert_eq!(fmt_macipr_str("foo", &args), Ok("foo\n".to_string()));
    }

    #[test]
    fn format_macaddr_mac_only() {
        let args = vec!["00:01:02:03:04:05".to_string()];
        assert_eq!(
            fmt_macipr_str("%m", &args),
            Ok("00:01:02:03:04:05\n".to_string())
        );
    }

    #[test]
    fn format_macaddr_mac_and_string() {
        let args = vec!["00:01:02:03:04:05".to_string()];
        assert_eq!(
            fmt_macipr_str("prefix %m", &args),
            Ok("prefix 00:01:02:03:04:05\n".to_string())
        );

        assert_eq!(
            fmt_macipr_str("%m postfix", &args),
            Ok("00:01:02:03:04:05 postfix\n".to_string())
        );

        assert_eq!(
            fmt_macipr_str("prefix %m postfix", &args),
            Ok("prefix 00:01:02:03:04:05 postfix\n".to_string())
        );
    }

    #[test]
    fn format_macaddr_multiple_macs() {
        let args = vec![
            "00:00:00:00:00:01".to_string(),
            "00:00:00:00:00:02".to_string(),
            "00:00:00:00:00:03".to_string(),
        ];
        assert_eq!(
            fmt_macipr_str("MAC %m, %m and %m", &args),
            Ok("MAC 00:00:00:00:00:01, 00:00:00:00:00:02 and 00:00:00:00:00:03\n".to_string())
        );
    }

    #[test]
    fn format_macaddr_range_one_mac() {
        let args = vec!["1-3".to_string()];
        assert_eq!(
            fmt_macipr_str("MAC=%m", &args),
            Ok("MAC=00:00:00:00:00:01\nMAC=00:00:00:00:00:02\nMAC=00:00:00:00:00:03\n".to_string())
        );
    }

    #[test]
    fn format_macaddr_range_multiple_macs() {
        let args = vec![
            "1-5".to_string(),
            "ff:ff:ff:ff:ff:00-ff:ff:ff:ff:ff:03".to_string(),
        ];
        assert_eq!(
            fmt_macipr_str("%m, %m", &args),
            Ok("\
00:00:00:00:00:01, ff:ff:ff:ff:ff:00
00:00:00:00:00:02, ff:ff:ff:ff:ff:01
00:00:00:00:00:03, ff:ff:ff:ff:ff:02
00:00:00:00:00:04, ff:ff:ff:ff:ff:03
00:00:00:00:00:05, ff:ff:ff:ff:ff:00
"
            .to_string())
        );
    }

    #[test]
    fn format_macaddr_invalid_mac_err() {
        let args = vec!["00:00:00-00:00:01".to_string()];
        assert_eq!(
            fmt_macipr_str("This is %m", &args),
            Err(FormatError {
                msg: "Invalid MAC address".to_string()
            })
        );
    }

    #[test]
    fn format_macaddr_insufficient_arg_err() {
        let args = vec!["00:00:00:00:00:01".to_string()];
        assert_eq!(
            fmt_macipr_str("This is %m", &vec![]),
            Err(FormatError {
                msg: "Insufficient number of arguments".to_string()
            })
        );

        assert_eq!(
            fmt_macipr_str("This is %m%m", &args),
            Err(FormatError {
                msg: "Insufficient number of arguments".to_string()
            })
        );
    }

    #[test]
    fn format_macaddr_unexpected_arg_err() {
        let args = vec!["00:00:00:00:00:01".to_string()];
        assert_eq!(
            fmt_macipr_str("This is it", &args),
            Err(FormatError {
                msg: "Unexpected argument".to_string()
            })
        );
    }

    #[test]
    fn format_ipv4addr_one_ipv4() {
        let args = vec!["192.168.0.1".to_string()];
        assert_eq!(
            fmt_macipr_str("This is %i", &args),
            Ok("This is 192.168.0.1\n".to_string())
        );
    }

    #[test]
    fn format_ipv4addr_invalid_ipv4_err() {
        let args = vec!["192.168.1".to_string()];
        assert_eq!(
            fmt_macipr_str("This is %i", &args),
            Err(FormatError {
                msg: "Invalid IPv4 address".to_string()
            })
        );
    }

    #[test]
    fn format_ipv6addr_one_ipv6() {
        let args = vec!["fe80::0100:0000:0000".to_string()];
        assert_eq!(
            fmt_macipr_str("This is %x", &args),
            Ok("This is fe80::100:0:0\n".to_string())
        );
    }

    #[test]
    fn format_ipv6addr_invalid_ipv6_err() {
        let args = vec!["fe80::0::0".to_string()];
        assert_eq!(
            fmt_macipr_str("This is %x", &args),
            Err(FormatError {
                msg: "Invalid IPv6 address".to_string()
            })
        );
    }

    #[test]
    fn format_ipv6fulladdr_one_ipv6() {
        let args = vec!["::1".to_string()];
        assert_eq!(
            fmt_macipr_str("This is %X", &args),
            Ok("This is 0000:0000:0000:0000:0000:0000:0000:0001\n".to_string())
        );
    }

    #[test]
    fn format_ipv6fulladdr_invalid_ipv6_err() {
        let args = vec!["fe80::0::0".to_string()];
        assert_eq!(
            fmt_macipr_str("This is %X", &args),
            Err(FormatError {
                msg: "Invalid IPv6 full address".to_string()
            })
        );
    }

    #[test]
    fn format_number_one_number() {
        let args = vec!["12345".to_string()];
        assert_eq!(
            fmt_macipr_str("This is %n", &args),
            Ok("This is 12345\n".to_string())
        );
    }

    #[test]
    fn format_number_invalid_number() {
        let args = vec!["-10".to_string()];
        assert_eq!(
            fmt_macipr_str("This is %n", &args),
            Err(FormatError {
                msg: "Invalid Number".to_string()
            })
        );
    }
}
