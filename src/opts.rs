use crate::logparser::LogFormat;
use clap::Clap;
use std::str::FromStr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LogFormatError {
    #[error("unknown format {0}")]
    UnknownFormat(String),
}

impl FromStr for LogFormat {
    type Err = LogFormatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "env_logger" => Ok(LogFormat {
                pattern: r"^\[(?P<ts>\S+) (?P<lvl>\S+) (?P<path>\S+)\] (?P<msg>.+)$",
                separator: "::",
            }),
            "pretty_env_logger" => Ok(LogFormat {
                pattern: r"^(?P<ts>) (?P<lvl>\S+) (?P<path>\S+)\s+> (?P<msg>.+)$",
                separator: "::",
            }),
            unknown => Err(LogFormatError::UnknownFormat(unknown.to_string())),
        }
    }
}

#[derive(Clap)]
pub struct Opts {
    #[clap(long)]
    pub name: Option<String>,
    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

#[derive(Clap)]
pub enum SubCommand {
    #[clap(about = "Reads logs from the stdin")]
    Stdin(Stdin),
    #[clap(about = "Reads logs from a file")]
    File(File),
    #[clap(about = "Polls Prometheus metrics endpoint")]
    Prometheus(Prometheus),
}

#[derive(Clap)]
pub struct Stdin {
    #[clap(long, default_value = "env_logger")]
    pub format: LogFormat,
}

#[derive(Clap)]
pub struct File {
    #[clap(long, default_value = "env_logger")]
    pub format: LogFormat,
    pub path: String,
}

#[derive(Clap)]
pub struct Prometheus {}
