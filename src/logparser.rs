use anyhow::Error;
use regex::Regex;
use rill::protocol::RillData;
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
}

impl LogParser {
    pub fn build(pattern: &str) -> Result<Self, Error> {
        let re = Regex::new(pattern)?;
        Ok(Self { re })
    }

    pub fn parse(&self, line: &str) -> Result<RillData, Error> {
        let cap = self.re.captures(&line).ok_or(LogParserError::NoMatches)?;
        let ts = cap.name("ts").ok_or(LogParserError::NoTimestamp)?;
        let timestamp = ts.as_str().to_owned();
        let lvl = cap.name("lvl").ok_or(LogParserError::NoLevel)?;
        let level = lvl.as_str().to_owned();
        let path = cap.name("path").ok_or(LogParserError::NoPath)?;
        let module = path.as_str().to_owned();
        let msg = cap.name("msg").ok_or(LogParserError::NoMessage)?;
        let message = msg.as_str().to_owned();
        let data = RillData::LogRecord {
            timestamp,
            level,
            module,
            message,
        };
        Ok(data)
    }
}
