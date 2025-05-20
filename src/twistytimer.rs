use std::path::Path;

use crate::models::{TwistyTimer, TwistyTimerError};

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
