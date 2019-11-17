use std::env::args;
use std::error::Error;
use std::io::{stdout, BufWriter};
use std::process::exit;

use macipr::format_macipr;

fn main() {
    if let Err(err) = macipr(args()) {
        eprintln!("{}", err);
        exit(1);
    }
}

fn macipr<I>(mut args: I) -> Result<(), Box<dyn Error>>
where
    I: Iterator<Item = String>,
{
    let format = match args.nth(1) {
        Some(format) => format,
        None => {
            return Err("usage: macipr FORMAT [MAC..]")?;
        }
    };
    let mut writer = BufWriter::new(stdout());
    Ok(format_macipr(&mut writer, &format, &args.collect())
        .map_err(|e| format!("macipr: {}", e))?)
}
