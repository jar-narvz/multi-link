//! UTC instants with a stable millisecond-resolution JSON representation.

use std::{fmt, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use time::{OffsetDateTime, UtcOffset, format_description::well_known::Rfc3339};

/// A UTC instant truncated to whole milliseconds at construction, so equality,
/// ordering, and hashing match its serialized representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp(OffsetDateTime);

impl Timestamp {
    pub fn from_epoch_ms(milliseconds: i64) -> Result<Self, time::error::ComponentRange> {
        OffsetDateTime::from_unix_timestamp_nanos(i128::from(milliseconds) * 1_000_000)
            .map(Self::from_instant)
    }

    pub fn parse(input: &str) -> Result<Self, time::error::Parse> {
        OffsetDateTime::parse(input, &Rfc3339).map(Self::from_instant)
    }

    fn from_instant(instant: OffsetDateTime) -> Self {
        let instant = instant.to_offset(UtcOffset::UTC);
        Self(
            instant
                .replace_nanosecond(instant.millisecond() as u32 * 1_000_000)
                .expect("whole milliseconds are a valid nanosecond value"),
        )
    }
}

impl FromStr for Timestamp {
    type Err = time::error::Parse;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Self::parse(input)
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let format = time::macros::format_description!(
            "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3]Z"
        );
        let text = self.0.format(format).map_err(|_| fmt::Error)?;
        formatter.write_str(&text)
    }
}

impl Serialize for Timestamp {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Timestamp {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let input = String::deserialize(deserializer)?;
        Self::parse(&input).map_err(serde::de::Error::custom)
    }
}
