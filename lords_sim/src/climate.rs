//! The sky above the world: the sun, the seasons, and the weather.
//!
//! None of it is kept anywhere. All of it follows from the [`Climate`] and the time, so that
//! the weather of any moment is the same however that moment is reached.

use std::f64::consts::TAU;
use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::rng::fixed_unit;
use crate::space::Length;
use crate::time::Timestamp;
use crate::time::gregorian::{Date, SECONDS_PER_DAY, SECONDS_PER_HOUR};

/// How far north the forest is, in degrees.
const LATITUDE: f64 = 62.0;

/// The tilt of the planet's axis, in degrees.
const AXIAL_TILT: f64 = 23.44;

const DAYS_PER_YEAR: f64 = 365.25;

/// The mean temperature of the whole year, and how far the warmest and the coldest time of
/// the year are from it, in degrees.
const YEARLY_MEAN: f64 = 3.0;
const YEARLY_SWING: f64 = 13.0;

/// The day of the year around which it is the warmest.
const WARMEST_DAY: f64 = 200.0;

/// How many days the ground and the water lag behind the air.
const GROUND_LAG: f64 = 15.0;

/// The hour of the day at which it is the warmest.
const WARMEST_HOUR: f64 = 15.0;

/// The temperature of a [`Climate::Still`] world.
const STILL_TEMPERATURE: Temperature = Temperature::from_tenths(150);

/// What kind of weather a world has.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Climate {
    /// An endless mild and bright summer day with no weather at all.
    #[default]
    Still,
    /// The weather of a northern forest, the same every time for the same `seed`.
    Boreal { seed: u64 },
}

impl Climate {
    /// Returns what the sky and the air are like at `time`.
    pub fn conditions(self, time: Timestamp) -> Conditions {
        let Self::Boreal { seed } = self else {
            return Conditions {
                temperature: STILL_TEMPERATURE,
                sky: Sky::Clear,
                wind: Wind::Calm,
                light: Light::Day,
                season: Season::Summer,
                snow: false,
                frozen: false,
            };
        };

        let date = Date::from_timestamp(time);
        let seconds = time.as_seconds();
        let hour = (seconds % SECONDS_PER_DAY) as f64 / SECONDS_PER_HOUR as f64;
        let day_of_year = f64::from(date.ordinal() - 1) + hour / 24.0;
        let days = seconds as f64 / SECONDS_PER_DAY as f64;
        let hours = seconds as f64 / SECONDS_PER_HOUR as f64;

        let yearly =
            |day: f64| YEARLY_MEAN + YEARLY_SWING * turn((day - WARMEST_DAY) / DAYS_PER_YEAR);
        let daily_swing = 3.5 + 2.0 * turn((day_of_year - 172.0) / DAYS_PER_YEAR);
        let degrees = yearly(day_of_year)
            + daily_swing * turn((hour - WARMEST_HOUR) / 24.0)
            + 5.0 * noise(seed, 0, days / 3.0)
            + 2.0 * noise(seed, 1, days / 0.7);
        let ground = yearly(day_of_year - GROUND_LAG) + 3.0 * noise(seed, 5, days / 10.0);

        let clouds = noise(seed, 2, hours / 9.0) + 0.3 * noise(seed, 3, hours / 2.5);
        let sky = if clouds > 0.55 {
            if degrees > 0.5 { Sky::Rain } else { Sky::Snow }
        } else if clouds > 0.1 {
            Sky::Overcast
        } else {
            Sky::Clear
        };

        let gusts = noise(seed, 4, hours / 6.0);
        let wind = if gusts > 0.5 {
            Wind::Windy
        } else if gusts > -0.2 {
            Wind::Breeze
        } else {
            Wind::Calm
        };

        Conditions {
            temperature: Temperature::from_tenths((degrees * 10.0).round() as i32),
            sky,
            wind,
            light: Light::of_sun_at(sun_elevation(day_of_year, hour)),
            season: Season::of(date),
            snow: ground < -1.0,
            frozen: ground < -3.0,
        }
    }
}

/// Returns the cosine of `turns` full turns.
fn turn(turns: f64) -> f64 {
    (TAU * turns).cos()
}

/// Returns how many degrees above the horizon the sun is.
fn sun_elevation(day_of_year: f64, hour: f64) -> f64 {
    let declination = (-AXIAL_TILT * turn((day_of_year + 10.0) / DAYS_PER_YEAR)).to_radians();
    let latitude = LATITUDE.to_radians();
    let from_noon = TAU * (hour - 12.0) / 24.0;

    (latitude.sin() * declination.sin() + latitude.cos() * declination.cos() * from_noon.cos())
        .asin()
        .to_degrees()
}

/// Returns a number from -1 to 1 that drifts smoothly as `at` grows, turning about once
/// for every whole number passed.
fn noise(seed: u64, channel: u64, at: f64) -> f64 {
    let whole = at.floor();
    let part = at - whole;
    let eased = part * part * (3.0 - 2.0 * part);
    let from = fixed_unit(seed, channel, whole as i64);
    let to = fixed_unit(seed, channel, whole as i64 + 1);

    2.0 * (from + (to - from) * eased) - 1.0
}

/// What the sky and the air are like at some moment.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Conditions {
    pub temperature: Temperature,
    pub sky: Sky,
    pub wind: Wind,
    pub light: Light,
    pub season: Season,
    /// Whether snow covers the ground.
    pub snow: bool,
    /// Whether ice covers the water.
    pub frozen: bool,
}

impl Conditions {
    /// Returns how fast food goes off, in hundredths of how fast it does on a mild day.
    pub fn spoiling_percent(&self) -> u32 {
        match self.temperature.as_tenths() {
            ..0 => 0,
            0..50 => 40,
            50..150 => 100,
            _ => 150,
        }
    }
}

/// A temperature in tenths of a degree Celsius.
#[derive(
    Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct Temperature(i32);

impl Temperature {
    pub const fn from_tenths(tenths: i32) -> Self {
        Self(tenths)
    }

    pub const fn from_degrees(degrees: i32) -> Self {
        Self(degrees * 10)
    }

    pub const fn as_tenths(self) -> i32 {
        self.0
    }

    /// Returns this temperature raised by `tenths` tenths of a degree.
    pub const fn warmer_by(self, tenths: i32) -> Self {
        Self(self.0 + tenths)
    }
}

/// Formats the temperature in whole degrees, such as `-4 °C`.
impl Display for Temperature {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let degrees = (f64::from(self.0) / 10.0).round() as i32;
        write!(f, "{degrees} °C")
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Sky {
    Clear,
    Overcast,
    Rain,
    Snow,
}

impl Sky {
    pub fn name(self) -> &'static str {
        match self {
            Self::Clear => "Clear",
            Self::Overcast => "Overcast",
            Self::Rain => "Rain",
            Self::Snow => "Snowfall",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Wind {
    Calm,
    Breeze,
    Windy,
}

impl Wind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Calm => "Calm",
            Self::Breeze => "Breeze",
            Self::Windy => "Windy",
        }
    }
}

/// How much there is to see by, from the least to the most.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Light {
    Dark,
    Dusk,
    Twilight,
    Day,
}

impl Light {
    fn of_sun_at(elevation: f64) -> Self {
        if elevation > 0.0 {
            Self::Day
        } else if elevation > -6.0 {
            Self::Twilight
        } else if elevation > -12.0 {
            Self::Dusk
        } else {
            Self::Dark
        }
    }

    /// Returns how far away things can be seen in this light.
    pub const fn sight(self) -> Length {
        match self {
            Self::Day => Length::from_meters(300),
            Self::Twilight => Length::from_meters(150),
            Self::Dusk => Length::from_meters(50),
            Self::Dark => Length::from_meters(12),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Day => "Daylight",
            Self::Twilight => "Twilight",
            Self::Dusk => "Dusk",
            Self::Dark => "Dark",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

impl Season {
    pub const fn of(date: Date) -> Self {
        match date.month() {
            3..=5 => Self::Spring,
            6..=8 => Self::Summer,
            9..=11 => Self::Autumn,
            _ => Self::Winter,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Spring => "Spring",
            Self::Summer => "Summer",
            Self::Autumn => "Autumn",
            Self::Winter => "Winter",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BOREAL: Climate = Climate::Boreal { seed: 11 };

    /// The moment `hour` hours into day `day` of the first year, days counted from zero.
    fn at(day: u64, hour: u64) -> Timestamp {
        Timestamp::from_seconds(day * SECONDS_PER_DAY + hour * SECONDS_PER_HOUR)
    }

    fn mean_temperature(days: std::ops::Range<u64>) -> i32 {
        let readings: Vec<i32> = days
            .flat_map(|day| (0..24).map(move |hour| at(day, hour)))
            .map(|time| BOREAL.conditions(time).temperature.as_tenths())
            .collect();

        readings.iter().sum::<i32>() / readings.len() as i32
    }

    #[test]
    fn still_climate_never_changes() {
        let conditions = Climate::Still.conditions(Timestamp::EPOCH);

        assert_eq!(conditions.temperature, Temperature::from_degrees(15));
        assert_eq!(conditions.light, Light::Day);
        assert!(!conditions.snow && !conditions.frozen);
        assert_eq!(Climate::Still.conditions(at(200, 3)), conditions);
    }

    #[test]
    fn weather_of_a_moment_is_always_the_same_for_the_same_seed() {
        let other = Climate::Boreal { seed: 12 };
        let moments = || (0..400).map(|day| at(day, day % 24));

        assert!(moments().all(|time| BOREAL.conditions(time) == BOREAL.conditions(time)));
        assert!(moments().any(|time| BOREAL.conditions(time) != other.conditions(time)));
    }

    #[test]
    fn summers_are_warm_and_winters_freezing() {
        let january = mean_temperature(0..31);
        let july = mean_temperature(181..212);

        assert!((-180..-40).contains(&january), "{january} in January");
        assert!((110..210).contains(&july), "{july} in July");
    }

    #[test]
    fn afternoons_are_warmer_than_nights_on_the_whole() {
        let mean_at = |hour| {
            (150..210)
                .map(|day| BOREAL.conditions(at(day, hour)).temperature.as_tenths())
                .sum::<i32>()
                / 60
        };

        assert!(mean_at(15) > mean_at(3) + 50);
    }

    #[test]
    fn midsummer_nights_are_never_dark_and_midwinter_days_are_short() {
        let light = |day, hour| BOREAL.conditions(at(day, hour)).light;
        let daylight_hours = |day| {
            (0..24)
                .filter(|hour| light(day, *hour) == Light::Day)
                .count()
        };

        assert_eq!(light(171, 12), Light::Day);
        assert_eq!(light(171, 0), Light::Twilight);
        assert_eq!(light(354, 0), Light::Dark);
        assert_eq!(light(354, 12), Light::Day);
        assert!(daylight_hours(171) >= 18, "{}", daylight_hours(171));
        assert!(daylight_hours(354) <= 6, "{}", daylight_hours(354));
    }

    #[test]
    fn snow_and_ice_come_for_the_winter_and_go_for_the_summer() {
        let conditions = |day| BOREAL.conditions(at(day, 12));

        assert!(conditions(20).snow && conditions(20).frozen);
        assert!(conditions(45).snow && conditions(45).frozen);
        assert!(!conditions(160).snow && !conditions(160).frozen);
        assert!(!conditions(240).snow && !conditions(240).frozen);
    }

    #[test]
    fn it_snows_only_when_it_is_cold_and_rains_only_when_it_is_not() {
        let mut rained = false;
        let mut snowed = false;

        for day in 0..365 {
            for hour in 0..24 {
                let conditions = BOREAL.conditions(at(day, hour));
                match conditions.sky {
                    Sky::Rain => {
                        rained = true;
                        assert!(conditions.temperature.as_tenths() > 0);
                    }
                    Sky::Snow => {
                        snowed = true;
                        assert!(conditions.temperature.as_tenths() <= 5);
                    }
                    Sky::Clear | Sky::Overcast => {}
                }
            }
        }

        assert!(rained && snowed);
    }

    #[test]
    fn seasons_follow_the_months() {
        let season = |day| Season::of(Date::from_timestamp(at(day, 0)));

        assert_eq!(season(0), Season::Winter);
        assert_eq!(season(59), Season::Spring);
        assert_eq!(season(151), Season::Summer);
        assert_eq!(season(243), Season::Autumn);
        assert_eq!(season(334), Season::Winter);
    }

    #[test]
    fn food_keeps_in_the_frost_and_goes_off_fastest_in_the_heat() {
        let spoiling = |degrees| {
            Conditions {
                temperature: Temperature::from_degrees(degrees),
                ..Climate::Still.conditions(Timestamp::EPOCH)
            }
            .spoiling_percent()
        };

        assert_eq!(spoiling(-5), 0);
        assert!(spoiling(2) < spoiling(10));
        assert!(spoiling(10) < spoiling(25));
    }

    #[test]
    fn temperature_is_written_in_whole_degrees() {
        assert_eq!(Temperature::from_tenths(-36).to_string(), "-4 °C");
        assert_eq!(Temperature::from_tenths(154).to_string(), "15 °C");
        assert_eq!(Temperature::from_degrees(0).to_string(), "0 °C");
    }
}
