#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]

pub const J1900_NAIF: f64 = 2_415_020.0;
pub const J2000_NAIF: f64 = 2_451_545.0;
/// `J1900_OFFSET` determines the offset in julian days between 01 Jan 1900 at midnight and the
/// Modified Julian Day at 17 November 1858.
/// NOTE: Julian days "start" at noon so that astronomical observations throughout the night
/// happen at the same Julian day. Note however that the Modified Julian Date (MJD) starts at
/// midnight, not noon, cf. <http://tycho.usno.navy.mil/mjd.html>.
pub const J1900_OFFSET: f64 = 15_020.0;
/// `J2000_OFFSET` determines the offset in julian days between 01 Jan 2000 at **noon** and the
/// Modified Julian Day at 17 November 1858.
pub const J2000_OFFSET: f64 = 51_544.5;
/// The Ephemeris Time epoch, in seconds
pub const ET_EPOCH_S: i64 = 3_155_716_800;
/// Modified Julian Date in seconds as defined [here](http://tycho.usno.navy.mil/mjd.html). MJD epoch is Modified Julian Day at 17 November 1858 at midnight.
pub const MJD_OFFSET: f64 = 2_400_000.5;
/// The JDE offset in days
pub const JDE_OFFSET_DAYS: f64 = J1900_OFFSET + MJD_OFFSET;
/// The JDE offset in seconds
pub const JDE_OFFSET_SECONDS: f64 = JDE_OFFSET_DAYS * SECONDS_PER_DAY;
/// `DAYS_PER_YEAR` corresponds to the number of days per year in the Julian calendar.
pub const DAYS_PER_YEAR: f64 = 365.25;
/// `DAYS_PER_CENTURY` corresponds to the number of days per centuy in the Julian calendar.
pub const DAYS_PER_CENTURY: f64 = 36525.0;
pub const DAYS_PER_CENTURY_I64: i64 = 36525;
/// `SECONDS_PER_MINUTE` defines the number of seconds per minute.
pub const SECONDS_PER_MINUTE: f64 = 60.0;
/// `SECONDS_PER_HOUR` defines the number of seconds per hour.
pub const SECONDS_PER_HOUR: f64 = 3_600.0;
/// `SECONDS_PER_DAY` defines the number of seconds per day.
pub const SECONDS_PER_DAY: f64 = 86_400.0;
pub const SECONDS_PER_DAY_I64: i64 = 86_400;
/// `SECONDS_PER_CENTURY` defines the number of seconds per century.
pub const SECONDS_PER_CENTURY: f64 = SECONDS_PER_DAY * DAYS_PER_CENTURY;
/// `SECONDS_PER_YEAR` corresponds to the number of seconds per julian year from [NAIF SPICE](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/jyear_c.html).
pub const SECONDS_PER_YEAR: f64 = 31_557_600.0;
pub const SECONDS_PER_YEAR_I64: i64 = 31_557_600;
/// `SECONDS_PER_TROPICAL_YEAR` corresponds to the number of seconds per tropical year from [NAIF SPICE](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/tyear_c.html).
pub const SECONDS_PER_TROPICAL_YEAR: f64 = 31_556_925.974_7;
/// `SECONDS_PER_SIDERAL_YEAR` corresponds to the number of seconds per sideral year from [NIST](https://www.nist.gov/pml/special-publication-811/nist-guide-si-appendix-b-conversion-factors/nist-guide-si-appendix-b9#TIME).
pub const SECONDS_PER_SIDERAL_YEAR: f64 = 31_558_150.0;
/// `SECONDS_GPS_TAI_OFFSET` is the number of seconds from the TAI epoch to the
/// GPS epoch (UTC midnight of January 6th 1980; cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>)
pub const SECONDS_GPS_TAI_OFFSET: f64 = 80.0 * SECONDS_PER_YEAR + 4.0 * SECONDS_PER_DAY + 19.0;
pub const SECONDS_GPS_TAI_OFFSET_I64: i64 =
    80 * SECONDS_PER_YEAR_I64 + 4 * SECONDS_PER_DAY_I64 + 19;
/// `DAYS_GPS_TAI_OFFSET` is the number of days from the TAI epoch to the GPS
/// epoch (UTC midnight of January 6th 1980; cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>)
pub const DAYS_GPS_TAI_OFFSET: f64 = SECONDS_GPS_TAI_OFFSET / SECONDS_PER_DAY;

/// The UNIX reference epoch of 1970-01-01.
pub const UNIX_REF_EPOCH: Epoch = Epoch::from_tai_duration(Duration {
    centuries: 0,
    nanoseconds: 2_208_988_800_000_000_000,
});

mod epoch;

pub use epoch::*;

mod duration;

pub use duration::*;

mod timeseries;
pub use timeseries::*;

pub mod prelude {
    pub use {Duration, Epoch, Freq, Frequencies, TimeSeries, TimeUnits, Unit};
}

extern crate num_traits;

#[cfg(feature = "std")]
extern crate serde;

#[cfg(feature = "std")]
extern crate core;

use core::convert;
use core::fmt;
use core::num::ParseIntError;
use core::str::FromStr;

#[cfg(feature = "std")]
extern crate regex;
#[cfg(feature = "std")]
extern crate serde_derive;
#[cfg(feature = "std")]
use std::error::Error;

/// Errors handles all oddities which may occur in this library.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Errors {
    /// Carry is returned when a provided function does not support time carry. For example,
    /// if a call to `Datetime::new` receives 60 seconds and there are only 59 seconds in the provided
    /// date time then a Carry Error is returned as the Result.
    Carry,
    /// ParseError is returned when a provided string could not be parsed and converted to the desired
    /// struct (e.g. Datetime).
    ParseError(ParsingErrors),
    /// Raised when trying to initialize an Epoch or Duration from its hi and lo values, but these overlap
    ConversionOverlapError(f64, f64),
    /// Raised if an overflow occured
    Overflow,
    /// Raised if the initialization from system time failed
    SystemTimeError,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ParsingErrors {
    ParseIntError,
    TimeSystem,
    ISO8601,
    UnknownFormat,
    UnknownUnit,
    UnsupportedTimeSystem,
}

impl fmt::Display for Errors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::Carry => write!(f, "a carry error (e.g. 61 seconds)"),
            Self::ParseError(kind) => write!(f, "ParseError: {:?}", kind),
            Self::ConversionOverlapError(hi, lo) => {
                write!(f, "hi and lo values overlap: {}, {}", hi, lo)
            }
            Self::Overflow => write!(
                f,
                "overflow occured when trying to convert Duration information"
            ),
            Self::SystemTimeError => write!(f, "std::time::SystemTime returned an error"),
        }
    }
}

impl convert::From<ParseIntError> for Errors {
    fn from(_: ParseIntError) -> Self {
        Errors::ParseError(ParsingErrors::ParseIntError)
    }
}

#[cfg(feature = "std")]
impl Error for Errors {}

/// Enum of the different time systems available
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TimeSystem {
    /// Ephemeris Time as defined by SPICE (slightly different from true TDB)
    ET,
    /// TAI is the representation of an Epoch internally
    TAI,
    /// Terrestrial Time (TT) (previously called Terrestrial Dynamical Time (TDT))
    TT,
    /// Dynamic Barycentric Time (TDB) (higher fidelity SPICE ephemeris time)
    TDB,
    /// Universal Coordinated Time
    UTC,
}

impl FromStr for TimeSystem {
    type Err = Errors;

    fn from_str(val: &str) -> Result<Self, Self::Err> {
        if val == "UTC" {
            Ok(TimeSystem::UTC)
        } else if val == "TT" {
            Ok(TimeSystem::TT)
        } else if val == "TAI" {
            Ok(TimeSystem::TAI)
        } else if val == "TDB" {
            Ok(TimeSystem::TDB)
        } else if val == "ET" {
            Ok(TimeSystem::ET)
        } else {
            Err(Errors::ParseError(ParsingErrors::TimeSystem))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Errors, ParsingErrors, TimeSystem};

    #[test]
    fn enum_eq() {
        // Check the equality compiles (if one compiles, then all asserts will work)
        assert!(Errors::Carry == Errors::Carry);
        assert!(ParsingErrors::ParseIntError == ParsingErrors::ParseIntError);
        assert!(TimeSystem::ET == TimeSystem::ET);
    }
}
