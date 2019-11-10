use std::convert::TryFrom;
use std::error::Error;
use std::fmt::{self, Display};

use crate::MacAddr;

#[derive(Debug, PartialEq)]
pub enum Format {
    MacAddr,
    RawString(String),
}

#[derive(Debug, PartialEq)]
enum FormatState {
    Normal,
    Percent,
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

pub fn format_macipr(fmt_str: &str, args: &Vec<String>) -> Result<String, FormatError> {
    let mut result = String::new();
    let mut offset = 0;
    for fmt in parse_format(fmt_str)? {
        match fmt {
            Format::RawString(s) => result.push_str(&s),
            Format::MacAddr => {
                if let Some(s) = args.get(offset) {
                    if let Ok(mac) = MacAddr::try_from(s.as_ref()) {
                        result.push_str(&format!("{}", mac));
                    } else {
                        return Err(FormatError {
                            msg: "Invalid MAC address".to_string(),
                        });
                    }
                } else {
                    return Err(FormatError {
                        msg: "Insufficient number of arguments".to_string(),
                    });
                }
                offset += 1;
            }
        }
    }
    if offset != args.len() {
        return Err(FormatError {
            msg: "Unexpected argument".to_string(),
        });
    }
    Ok(result)
}

fn parse_format(fmt_str: &str) -> Result<Vec<Format>, FormatError> {
    let mut fmts = vec![];
    let mut buf = String::new();
    let mut state = FormatState::Normal;
    for c in fmt_str.chars() {
        if c == '%' {
            state = FormatState::Percent;
            continue;
        }
        if state == FormatState::Percent {
            state = FormatState::Normal;
            match c {
                '%' => buf.push('%'),
                'm' => {
                    if !buf.is_empty() {
                        fmts.push(Format::RawString(buf));
                        buf = String::new();
                    }
                    fmts.push(Format::MacAddr);
                }
                _ => {
                    return Err(FormatError {
                        msg: "Unexpected character after %".to_string(),
                    });
                }
            }
        } else {
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
    fn parse_format_macaddr() {
        assert_eq!(parse_format("%m"), Ok(vec![Format::MacAddr]));
    }

    #[test]
    fn parse_format_error() {
        assert_eq!(
            parse_format("%k"),
            Err(FormatError {
                msg: "Unexpected character after %".to_string()
            })
        );
    }

    #[test]
    fn format_macaddr_mac_only() {
        let args = vec!["00:01:02:03:04:05".to_string()];
        assert_eq!(
            format_macipr("%m", &args),
            Ok("00:01:02:03:04:05".to_string())
        );
    }

    #[test]
    fn format_macaddr_mac_and_string() {
        let args = vec!["00:01:02:03:04:05".to_string()];
        assert_eq!(
            format_macipr("prefix %m", &args),
            Ok("prefix 00:01:02:03:04:05".to_string())
        );

        assert_eq!(
            format_macipr("%m postfix", &args),
            Ok("00:01:02:03:04:05 postfix".to_string())
        );

        assert_eq!(
            format_macipr("prefix %m postfix", &args),
            Ok("prefix 00:01:02:03:04:05 postfix".to_string())
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
            format_macipr("MAC %m, %m and %m", &args),
            Ok("MAC 00:00:00:00:00:01, 00:00:00:00:00:02 and 00:00:00:00:00:03".to_string())
        );
    }

    #[test]
    fn format_macaddr_invalid_mac_err() {
        let args = vec!["00:00:00-00:00:01".to_string()];
        assert_eq!(
            format_macipr("This is %m", &args),
            Err(FormatError {
                msg: "Invalid MAC address".to_string()
            })
        );
    }

    #[test]
    fn format_macaddr_insufficient_arg_err() {
        let args = vec!["00:00:00:00:00:01".to_string()];
        assert_eq!(
            format_macipr("This is %m", &vec![]),
            Err(FormatError {
                msg: "Insufficient number of arguments".to_string()
            })
        );

        assert_eq!(
            format_macipr("This is %m%m", &args),
            Err(FormatError {
                msg: "Insufficient number of arguments".to_string()
            })
        );
    }

    #[test]
    fn format_macaddr_unexpected_arg_err() {
        let args = vec!["00:00:00:00:00:01".to_string()];
        assert_eq!(
            format_macipr("This is it", &args),
            Err(FormatError {
                msg: "Unexpected argument".to_string()
            })
        );
    }
}
