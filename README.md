# Rolling

rolling is a simple command line tool that outputs log files in a file-rotating directory.
The tool was designed to adhere by the [Unix Philosophy](https://en.wikipedia.org/wiki/Unix_philosophy)
as much as possible. So all it does it write it's standard input into a directory with file rotation.

it will automatically delete the oldest log file in the directory to strictly enforce a maximum amount
of space utilized by the directory. Use it to ensure that your logs never consume more than what you want of your disk space.


### Usage

For convenience, it also outputs it's input verbatim so you can compose `rolling` easily via unix pipes.

```man
USAGE:
    rolling [FLAGS] [OPTIONS] <dir>

ARGS:
    <dir>    the output directory where the log files are rotated

FLAGS:
    -h, --help       Prints help information
    -s, --silent     whether output input to standard out
    -V, --version    Prints version information

OPTIONS:
    -b, --max-bytes-per-file <bytes>       max number of bytes in each log file [default: 25M]
    -m, --max-log-files <max-log-files>    max number of log files in the directory [default: 10]
```
