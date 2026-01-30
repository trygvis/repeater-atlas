use diesel::deserialize::{self, FromSql};
use diesel::pg::{Pg, PgValue};
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::BigInt;
use diesel::{AsExpression, FromSqlRow};
use serde::Serialize;
use std::fmt;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, AsExpression, FromSqlRow)]
#[diesel(sql_type = BigInt)]
pub struct Frequency(i64);

impl Serialize for Frequency {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_i64(self.hz())
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum FrequencyError {
    NegativeHz(i64),
    Overflow { base_hz: i64, offset_hz: i64 },
}

impl fmt::Display for FrequencyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FrequencyError::NegativeHz(hz) => write!(f, "frequency must be >= 0 Hz, got {hz}"),
            FrequencyError::Overflow { base_hz, offset_hz } => write!(
                f,
                "frequency overflow when adding offset: {base_hz} + {offset_hz}"
            ),
        }
    }
}

impl std::error::Error for FrequencyError {}

impl Frequency {
    pub fn new_hz(hz: i64) -> Result<Self, FrequencyError> {
        if hz < 0 {
            return Err(FrequencyError::NegativeHz(hz));
        }
        Ok(Self(hz))
    }

    pub fn band_label(&self) -> &'static str {
        let hz = self.hz();
        if hz < 30_000_000 {
            "HF"
        } else if hz < 300_000_000 {
            "VHF"
        } else if hz < 3_000_000_000 {
            "UHF"
        } else {
            "SHF"
        }
    }

    pub fn contained_in(&self, range: std::ops::Range<i64>) -> bool {
        range.contains(&self.0)
    }

    pub fn offset(&self, offset_hz: i64) -> Result<Self, FrequencyError> {
        let hz = self
            .0
            .checked_add(offset_hz)
            .ok_or(FrequencyError::Overflow {
                base_hz: self.0,
                offset_hz,
            })?;
        Self::new_hz(hz)
    }

    pub fn hz(&self) -> i64 {
        self.0
    }
}

impl fmt::Display for Frequency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hz = self.hz();

        // Prefer the largest unit that yields a <1000 value (matching `##0...`).
        // Fall back to Hz for very small values.
        let (factor, unit, max_decimals) = {
            let candidates = [
                (1_000_000_000_i64, "GHz", 9_usize),
                (1_000_000_i64, "MHz", 6_usize),
                (1_000_i64, "kHz", 3_usize),
                (1_i64, "Hz", 0_usize),
            ];

            let mut selected = candidates[candidates.len() - 1];
            for candidate in candidates {
                let (factor, unit, max_decimals) = candidate;
                if hz >= factor {
                    let whole = hz / factor;
                    if whole < 1000 || factor == 1_000_000_000 {
                        selected = (factor, unit, max_decimals);
                        break;
                    }
                }
            }
            selected
        };

        let whole = hz / factor;
        let remainder = hz % factor;

        let decimals = if max_decimals == 0 {
            "000".to_string()
        } else {
            let full = format!("{remainder:0width$}", width = max_decimals);
            let trimmed = full.trim_end_matches('0');
            if trimmed.len() >= 3 {
                trimmed.to_string()
            } else {
                full[..3].to_string()
            }
        };

        write!(f, "{whole}.{decimals} {unit}")
    }
}

impl ToSql<BigInt, Pg> for Frequency {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        <i64 as ToSql<BigInt, Pg>>::to_sql(&self.0, out)
    }
}

impl FromSql<BigInt, Pg> for Frequency {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        let hz = <i64 as FromSql<BigInt, Pg>>::from_sql(bytes)?;
        Self::new_hz(hz).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::Range;

    #[test]
    fn formats_mhz_with_three_decimals() {
        let f = Frequency::new_hz(145_700_000).unwrap();
        assert_eq!(f.to_string(), "145.700 MHz");
    }

    #[test]
    fn formats_mhz_half_megahertz() {
        let f = Frequency::new_hz(145_500_000).unwrap();
        assert_eq!(f.to_string(), "145.500 MHz");
    }

    #[test]
    fn formats_with_more_decimals_when_needed() {
        let f = Frequency::new_hz(145_712_500).unwrap();
        assert_eq!(f.to_string(), "145.7125 MHz");
    }

    #[test]
    fn formats_khz() {
        let f = Frequency::new_hz(12_345).unwrap();
        assert_eq!(f.to_string(), "12.345 kHz");
    }

    #[test]
    fn formats_ghz() {
        let f = Frequency::new_hz(1_297_000_000).unwrap();
        assert_eq!(f.to_string(), "1.297 GHz");
    }

    #[test]
    fn formats_ghz_one_point_two() {
        let f = Frequency::new_hz(1_200_000_000).unwrap();
        assert_eq!(f.to_string(), "1.200 GHz");
    }

    #[test]
    fn formats_hz() {
        let f = Frequency::new_hz(500).unwrap();
        assert_eq!(f.to_string(), "500.000 Hz");
    }

    #[test]
    fn rejects_negative() {
        assert!(Frequency::new_hz(-1).is_err());
    }

    #[test]
    fn offset_adds_hz() {
        let f = Frequency::new_hz(145_500_000).unwrap();
        assert_eq!(f.offset(600_000).unwrap().hz(), 146_100_000);
    }

    #[test]
    fn offset_rejects_negative_result() {
        let f = Frequency::new_hz(100).unwrap();
        assert!(matches!(f.offset(-101), Err(FrequencyError::NegativeHz(_))));
    }

    #[test]
    fn offset_rejects_overflow() {
        let f = Frequency::new_hz(i64::MAX).unwrap();
        assert!(matches!(f.offset(1), Err(FrequencyError::Overflow { .. })));
    }

    #[test]
    fn contained_in_half_open_range() {
        let f = Frequency::new_hz(30_000_000).unwrap();
        let r: Range<i64> = 30_000_000..300_000_000;
        assert!(f.contained_in(r));
        assert!(!f.contained_in(0..30_000_000));
    }
}
