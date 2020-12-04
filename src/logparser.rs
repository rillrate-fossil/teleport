use anyhow::Error;
use regex::Regex;
use rill::protocol::{EntryId, Path, RillData};
use thiserror::Error;

#[derive(Error, Debug)]
enum LogParserError {
    #[error("no matches")]
    NoMatches,
    #[error("no timestamp")]
    NoTimestamp,
    #[error("no level")]
    NoLevel,
    #[error("no path")]
    NoPath,
    #[error("no message")]
    NoMessage,
}

pub struct LogParser {
    re: Regex,
    separator: String,
}

pub struct LogRecord {
    pub path: Path,
    pub data: RillData,
}

impl LogParser {
    pub fn build(pattern: &str, separator: &str) -> Result<Self, Error> {
        let re = Regex::new(pattern)?;
        let separator = separator.to_string();
        Ok(Self { re, separator })
    }

    pub fn parse(&self, line: &str) -> Result<LogRecord, Error> {
        let cap = self.re.captures(&line).ok_or(LogParserError::NoMatches)?;

        let ts = cap.name("ts").ok_or(LogParserError::NoTimestamp)?;
        let timestamp = ts.as_str().to_owned();
        let msg = cap.name("msg").ok_or(LogParserError::NoMessage)?;
        let message = msg.as_str().to_owned();
        let data = RillData::LogRecord { timestamp, message };

        let lvl = cap.name("lvl").ok_or(LogParserError::NoLevel)?;
        let path = cap.name("path").ok_or(LogParserError::NoPath)?;
        let mut path: Vec<_> = path
            .as_str()
            .split(&self.separator)
            .map(EntryId::from)
            .collect();
        let level = EntryId::from(lvl.as_str().to_lowercase());
        path.push(level);

        let record = LogRecord {
            path: Path::from(path),
            data,
        };

        Ok(record)
    }
}
