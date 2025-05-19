use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Deserializer};
use std::{fmt, path::Path, str::FromStr};
use thiserror::Error;

/// Represents the types of puzzles supported by TwistyTimer
#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub enum TwistyTimerPuzzles {
    Cube222,
    Cube333,
    Cube444,
    Cube555,
    Cube666,
    Cube777,
    CubeSq1,
    CubeSkewb,
    CubeClock,
    CubePyra,
    CubeMega,
    Unknown(String),
}

impl fmt::Display for TwistyTimerPuzzles {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cube222 => write!(f, "222"),
            Self::Cube333 => write!(f, "333"),
            Self::Cube444 => write!(f, "444"),
            Self::Cube555 => write!(f, "555"),
            Self::Cube666 => write!(f, "666"),
            Self::Cube777 => write!(f, "777"),
            Self::CubeSq1 => write!(f, "sq1"),
            Self::CubeSkewb => write!(f, "skewb"),
            Self::CubeClock => write!(f, "clock"),
            Self::CubePyra => write!(f, "pyra"),
            Self::CubeMega => write!(f, "mega"),
            Self::Unknown(s) => write!(f, "unknown:{}", s),
        }
    }
}

impl FromStr for TwistyTimerPuzzles {
    type Err = TwistyTimerError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim().to_lowercase();

        match s.as_str() {
            "222" => Ok(Self::Cube222),
            "333" => Ok(Self::Cube333),
            "444" => Ok(Self::Cube444),
            "555" => Ok(Self::Cube555),
            "666" => Ok(Self::Cube666),
            "777" => Ok(Self::Cube777),
            "sq1" => Ok(Self::CubeSq1),
            "skewb" => Ok(Self::CubeSkewb),
            "clock" => Ok(Self::CubeClock),
            "pyra" => Ok(Self::CubePyra),
            "mega" => Ok(Self::CubeMega),
            _ => {
                if s.is_empty() {
                    Err(TwistyTimerError::EmptyPuzzleType)
                } else {
                    Ok(Self::Unknown(s.to_string()))
                }
            }
        }
    }
}

impl<'de> Deserialize<'de> for TwistyTimerPuzzles {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        TwistyTimerPuzzles::from_str(&s).map_err(serde::de::Error::custom)
    }
}

/// Defines specific errors that can occur during TwistyTimer operations
#[derive(Error, Debug)]
pub enum TwistyTimerError {
    #[error("Empty puzzle type")]
    EmptyPuzzleType,

    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(String),

    #[error("Failed to parse timestamp '{0}': {1}")]
    TimestampParseError(String, #[source] std::num::ParseIntError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("CSV error: {0}")]
    CsvError(#[from] csv::Error),

    #[error("CSV error at record {0}: {1}")]
    CsvRecordError(usize, #[source] csv::Error),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid time value: {0}")]
    InvalidTimeValue(String),
}

/// Represents a single solve record from TwistyTimer
#[derive(Debug, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct TwistyTimer {
    pub puzzle: TwistyTimerPuzzles,
    pub category: String,
    #[serde(deserialize_with = "deserialize_time")]
    pub time: u32,
    #[serde(deserialize_with = "deserialize_date")]
    pub date: DateTime<Utc>,
    pub scramble: String,
    pub penalty: String,
    #[serde(default)]
    pub comment: String,
}

fn deserialize_date<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    if s.trim().is_empty() {
        return Err(serde::de::Error::custom(TwistyTimerError::MissingField(
            "date".to_string(),
        )));
    }

    let millis = s.parse::<i64>().map_err(|e| {
        serde::de::Error::custom(TwistyTimerError::TimestampParseError(s.clone(), e))
    })?;

    if !(0..=i64::MAX / 1_000_000).contains(&millis) {
        return Err(serde::de::Error::custom(
            TwistyTimerError::InvalidTimestamp(s),
        ));
    }

    let secs = millis / 1000;
    let nsecs = (millis % 1000) * 1_000_000;

    Utc.timestamp_opt(secs, nsecs as u32)
        .single()
        .ok_or_else(|| serde::de::Error::custom(TwistyTimerError::InvalidTimestamp(s)))
}

fn deserialize_time<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    if s.trim().is_empty() {
        return Err(serde::de::Error::custom(TwistyTimerError::MissingField(
            "time".to_string(),
        )));
    }

    let time = s
        .parse::<u32>()
        .map_err(|_| serde::de::Error::custom(TwistyTimerError::InvalidTimeValue(s.clone())))?;

    const MAX_REASONABLE_TIME: u32 = 24 * 60 * 60 * 1000;
    if time > MAX_REASONABLE_TIME {
        return Err(serde::de::Error::custom(
            TwistyTimerError::InvalidTimeValue(format!("{} (unreasonably large)", s)),
        ));
    }

    Ok(time)
}

#[derive(Debug, Clone, Default)]
pub struct ParseConfig {
    /// Skip records with invalid values instead of returning an error
    pub skip_invalid_records: bool,
}

/// Parses TwistyTimer records from a CSV file
pub fn parse_twistytimer<P: AsRef<Path>>(
    path: P,
    config: Option<ParseConfig>,
) -> Result<Vec<TwistyTimer>, TwistyTimerError> {
    let config = config.unwrap_or_default();
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(path)?;

    let mut records = Vec::new();

    for (idx, result) in reader.deserialize::<TwistyTimer>().enumerate() {
        match result {
            Ok(record) => {
                records.push(record);
            }
            Err(err) => {
                if !config.skip_invalid_records {
                    return Err(TwistyTimerError::CsvRecordError(idx + 1, err));
                }
            }
        }
    }

    Ok(records)
}
