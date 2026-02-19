use serde::Serialize;
use std::fmt;

use crate::MaidenheadLocator;

/// Geographic point represented as latitude/longitude in decimal degrees.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Point {
    pub latitude: f64,
    pub longitude: f64,
}

impl Point {
    pub const fn from_latlon(latitude: f64, longitude: f64) -> Self {
        Self {
            latitude,
            longitude,
        }
    }

    pub fn from_optional(latitude: Option<f64>, longitude: Option<f64>) -> Option<Self> {
        match (latitude, longitude) {
            (Some(latitude), Some(longitude)) => Some(Self::from_latlon(latitude, longitude)),
            _ => None,
        }
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}", self.latitude, self.longitude)
    }
}

impl From<MaidenheadLocator> for Point {
    fn from(value: MaidenheadLocator) -> Self {
        let (longitude, latitude) = value.longlat();
        Self::from_latlon(latitude, longitude)
    }
}

#[cfg(test)]
mod tests {
    use super::Point;

    #[test]
    fn from_optional_requires_both_coordinates() {
        assert_eq!(
            Point::from_optional(Some(63.0), Some(10.0)),
            Some(Point::from_latlon(63.0, 10.0))
        );
        assert_eq!(Point::from_optional(Some(63.0), None), None);
        assert_eq!(Point::from_optional(None, Some(10.0)), None);
        assert_eq!(Point::from_optional(None, None), None);
    }
}
