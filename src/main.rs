use clap::Clap;
use std::fs::create_dir_all;
use std::io::{self, Write};

mod error;
mod rolling_writer;

use crate::{
    error::{Result, RollingError},
    rolling_writer::RollingWriter,
};

fn main() -> Result<()> {
    let mut input = String::new();
    let opts = Opts::parse();

    create_dir_all(format!("{}", opts.dir))?;
    let mut writer = RollingWriter::new(
        opts.dir,
        parse_bytes(opts.bytes.as_str())?,
        opts.max_log_files,
    )?;

    while io::stdin().read_line(&mut input)? != 0 {
        writer.write(input.as_bytes())?;
        if !opts.silent {
            print!("{}", input.as_str())
        }
    }
    Ok(())
}

fn parse_bytes(mut bytes_string: &str) -> Result<usize> {
    let mut multiple: usize = 1;
    if let Some(val) = bytes_string.find(|c: char| c.is_alphabetic()) {
        multiple = match &bytes_string[val..] {
            "M" | "m" => 1024 * 1024,
            "G" | "g" => 1024 * 1024 * 1024,
            "K" | "k" => 1024,
            _ => Err(RollingError::ParseError)?,
        };
        bytes_string = &bytes_string[..val];
    }
    let value: f64 = bytes_string.parse().map_err(|_| RollingError::ParseError)?;

    Ok((value * (multiple as f64)).floor() as usize)
}

#[derive(Clap)]
#[clap(version = "0.1", author = "Shalom Yiblet <shalom.yiblet@gmail.com>")]
struct Opts {
    /// the output directory where the log files are rotated
    dir: String,

    #[clap(short, long, about = "whether output input to standard out")]
    silent: bool,

    #[clap(
        short = "b",
        long = "max-bytes-per-file",
        default_value = "25M",
        about = "max number of bytes in each log file"
    )]
    bytes: String,

    #[clap(
        short,
        long,
        default_value = "10",
        about = "max number of log files in the directory"
    )]
    max_log_files: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_test() {
        assert_eq!(parse_bytes("5M").unwrap(), 1024 * 1024 * 5);
        assert_eq!(parse_bytes("4M").unwrap(), 1024 * 1024 * 4);
    }
}
