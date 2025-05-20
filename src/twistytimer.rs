use std::path::Path;

use crate::models::{TwistyTimer, TwistyTimerError};

/// Parses TwistyTimer records from a CSV file
pub fn parse_twistytimer<P: AsRef<Path>>(path: P) -> Result<Vec<TwistyTimer>, TwistyTimerError> {
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
                return Err(TwistyTimerError::CsvRecordError(idx + 1, err));
            }
        }
    }

    Ok(records)
}
