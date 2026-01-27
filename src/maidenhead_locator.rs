use diesel::deserialize::{self, FromSql};
use diesel::pg::{Pg, PgValue};
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::sql_types::Text;
use diesel::{AsExpression, FromSqlRow};
use std::fmt;
use std::io::Write;
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq, Hash, AsExpression, FromSqlRow)]
#[diesel(sql_type = Text)]
pub struct MaidenheadLocator(String);

impl MaidenheadLocator {
    pub fn new(value: impl AsRef<str>) -> Result<Self, maidenhead::MHError> {
        let value = value.as_ref().trim();
        let (longitude, latitude) = maidenhead::grid_to_longlat(value)?;
        let canonical = maidenhead::longlat_to_grid(longitude, latitude, value.len())?;
        Ok(Self(canonical))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the center of the locator square.
    pub fn longlat(&self) -> (f64, f64) {
        // `MaidenheadLocator::new` guarantees validity and canonical format.
        maidenhead::grid_to_longlat(self.as_str()).expect("MaidenheadLocator must be valid")
    }
}

impl fmt::Display for MaidenheadLocator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for MaidenheadLocator {
    type Err = maidenhead::MHError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl ToSql<Text, Pg> for MaidenheadLocator {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        out.write_all(self.as_str().as_bytes())?;
        Ok(IsNull::No)
    }
}

impl FromSql<Text, Pg> for MaidenheadLocator {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        let value = <String as FromSql<Text, Pg>>::from_sql(bytes)?;
        Self::new(&value).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_canonicalizes_case() {
        let locator = MaidenheadLocator::new("jp53FI").unwrap();
        assert_eq!(locator.to_string(), "JP53fi");
    }

    #[test]
    fn rejects_invalid_length() {
        let err = MaidenheadLocator::new("JP53F").unwrap_err();
        assert!(matches!(err, maidenhead::MHError::InvalidGridLength(_)));
    }
}
