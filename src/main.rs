use clap::Clap;
use std::fs::{create_dir_all, File};
use std::{
    collections::VecDeque,
    io::{self, Write},
};
use thiserror::Error;

#[derive(Debug, Error)]
enum RollingError {
    #[error("couldn't parse input")]
    ParseError,
    #[error("failed to write result")]
    WriterResult(#[from] std::io::Error),
}

type Result<T> = std::result::Result<T, RollingError>;

struct RollingWriter {
    files: VecDeque<(usize, String)>,
    roll_dir: String,
    num_rolls: usize,
    max_files: usize,
    max_bytes_written: usize,
    cur_bytes_written: usize,
    cur_file: File,
}

fn get_id(file: &str) -> Option<usize> {
    let prefix = &file[..file.len() - ".log".len()];
    prefix.parse().ok()
}

fn get_new_file(dir: &str, id: usize) -> (String, String) {
    (format!("{}/{:03}.log", dir, id), format!("{:03}.log", id))
}

impl RollingWriter {
    pub fn new(roll_dir: String, max_bytes_written: usize, max_files: usize) -> Result<Self> {
        let mut dir_entries: Vec<(usize, String)> = std::fs::read_dir(roll_dir.as_str())?
            .filter(|entry| entry.is_ok())
            .map(|entry| -> Result<_> {
                let dir_entry = entry.unwrap();
                let file_name = dir_entry
                    .path()
                    .file_name()
                    .and_then(|path| path.to_owned().into_string().ok())
                    .ok_or_else(|| {
                        io::Error::new(io::ErrorKind::NotFound, "file name is missing")
                    })?;
                Ok((
                    get_id(file_name.as_str()).ok_or_else(|| {
                        io::Error::new(io::ErrorKind::NotFound, "file name is missing")
                    })?,
                    file_name,
                ))
            })
            .filter(|path| path.is_ok())
            .map(|path| (path.unwrap()))
            .collect();

        dir_entries.sort();
        let files: VecDeque<(usize, String)> = dir_entries.into_iter().collect();
        let mut num_rolls = files.back().map_or(0, |val| val.0 + 1);
        let (new_file, new_file_name) = get_new_file(roll_dir.as_str(), num_rolls);
        num_rolls += 1;
        let file = File::create(new_file)?;

        let mut output = Self {
            files,
            roll_dir,
            num_rolls,
            max_files,
            max_bytes_written,
            cur_bytes_written: 0,
            cur_file: file,
        };
        output.add_file(new_file_name.as_str(), num_rolls - 1)?;

        Ok(output)
    }

    fn add_file(&mut self, file: &str, roll_id: usize) -> io::Result<()> {
        while self.files.len() > self.max_files {
            let removed =
                self.files.pop_front().map(|val| val.1).ok_or_else(|| {
                    io::Error::new(io::ErrorKind::NotFound, "files already empty")
                })?;
            std::fs::remove_file(format!("{}/{}", self.roll_dir, removed))?;
        }

        Ok(self.files.push_back((roll_id, file.to_owned())))
    }

    fn roll(&mut self) -> io::Result<()> {
        self.cur_file.flush()?;
        self.cur_bytes_written = 0;
        let (new_file, new_file_name) = get_new_file(self.roll_dir.as_str(), self.num_rolls);
        self.add_file(new_file_name.as_str(), self.num_rolls)?;
        self.num_rolls += 1;
        self.cur_file = File::create(new_file)?;
        Ok(())
    }
}

impl Write for RollingWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if self.cur_bytes_written + buf.len() > self.max_bytes_written {
            self.roll()?;
        }

        self.cur_bytes_written += buf.len();
        self.cur_file.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.cur_file.flush()
    }
}

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
    /// Sets a custom config file. Could have been an Option<T> with no default too
    dir: String,

    #[clap(short, about = "whether output input to standard out")]
    silent: bool,

    #[clap(
        short = "b",
        long,
        default_value = "25M",
        about = "max number of bytes"
    )]
    bytes: String,

    #[clap(short, long, default_value = "10", about = "max number of log files")]
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

    #[test]
    fn get_id_test() {
        assert_eq!(get_id("01.log"), Some(1));
        assert_eq!(get_id("2311.log"), Some(2311));
    }
}
