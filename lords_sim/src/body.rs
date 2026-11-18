//! What a body needs to stay alive, and what becomes of it when it goes without.
//!
//! A body changes a minute at a time, see [`Body::pass_minute`].

use serde::{Deserialize, Serialize};

use crate::climate::Temperature;
use crate::item::Food;
use crate::time::Timestamp;

/// The food energy a body holds when full, in calories.
const ENERGY_FULL: u32 = 3_000_000;

/// The water a body has to spare when it has drunk its fill, in tenths of a milliliter.
const WATER_FULL: u32 = 30_000;

/// The sleep owed at which a body falls asleep wherever it is, in seconds.
///
/// A waking hour at rest adds half an hour to the sleep owed, so this is two days awake.
pub const FATIGUE_LIMIT: u32 = 24 * 3_600;

const TIRED: u32 = 8 * 3_600;
const EXHAUSTED: u32 = 18 * 3_600;

/// The seconds of sleep owed that a minute of sleep pays back when cold, on the ground, and
/// on a bed.
const SLEEP: (u32, u32, u32) = (35, 55, 65);

/// The temperature of a healthy body, in millionths of a degree.
const CORE_NORMAL: i32 = 37_000_000;
const CORE_CHILLED: i32 = 36_300_000;
const CORE_HYPOTHERMIC: i32 = 35_000_000;
const CORE_FREEZING: i32 = 32_000_000;
const CORE_DEAD: i32 = 28_000_000;

/// The temperature below which nobody stays asleep.
const CORE_WAKING: i32 = 35_600_000;

/// How warm it has to feel for a body at rest to lose no heat.
const COMFORT: Temperature = Temperature::from_degrees(18);

/// How many tenths of a degree short of [`COMFORT`] a body makes up for by itself, at the
/// cost of burning more energy.
const SHIVER_RANGE: i32 = 50;

/// How many millionths of a degree a body cools in a minute for each tenth of a degree it
/// feels colder than it can make up for.
const COOLING: i32 = 74;

const HEALTH_FULL: u32 = 1_000_000;

/// The health lost in a minute to each thing that harms.
const STARVING_HARM: u32 = 28;
const PARCHED_HARM: u32 = 231;
const EXHAUSTED_HARM: u32 = 50;
const FREEZING_HARM: u32 = 5_500;
const SICK_HARM: u32 = 25;

/// The health gained in a minute by a body that lacks nothing, awake and asleep.
const HEALING: (u32, u32) = (70, 110);

/// How hard a body is working.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Effort {
    Sleeping,
    Resting,
    Walking,
    Working,
}

impl Effort {
    /// Returns the calories burned in a minute.
    const fn energy(self) -> u32 {
        match self {
            Self::Sleeping => 1_100,
            Self::Resting => 1_400,
            Self::Walking => 4_200,
            Self::Working => 5_500,
        }
    }

    /// Returns the tenths of a milliliter of water lost in a minute.
    const fn water(self) -> u32 {
        match self {
            Self::Sleeping => 12,
            Self::Resting => 17,
            Self::Walking => 30,
            Self::Working => 38,
        }
    }

    /// Returns the seconds of sleep that a minute adds to those owed.
    const fn fatigue(self) -> u32 {
        match self {
            Self::Sleeping => 0,
            Self::Resting => 30,
            Self::Walking => 34,
            Self::Working => 38,
        }
    }

    /// Returns how many tenths of a degree warmer the effort makes it feel.
    pub const fn warmth(self) -> i32 {
        match self {
            Self::Sleeping => -20,
            Self::Resting => 0,
            Self::Walking => 40,
            Self::Working => 60,
        }
    }
}

/// Something wrong with a body, the milder of each kind before the graver.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Ailment {
    Hungry,
    Starving,
    Thirsty,
    Parched,
    Tired,
    Exhausted,
    Chilled,
    Hypothermic,
    Freezing,
    Sick,
}

impl Ailment {
    pub fn name(self) -> &'static str {
        match self {
            Self::Hungry => "Hungry",
            Self::Starving => "Starving",
            Self::Thirsty => "Thirsty",
            Self::Parched => "Parched",
            Self::Tired => "Tired",
            Self::Exhausted => "Exhausted",
            Self::Chilled => "Chilled",
            Self::Hypothermic => "Hypothermic",
            Self::Freezing => "Freezing to death",
            Self::Sick => "Sick",
        }
    }

    /// Returns whether the ailment kills if nothing is done about it.
    pub const fn is_grave(self) -> bool {
        matches!(
            self,
            Self::Starving | Self::Parched | Self::Exhausted | Self::Hypothermic | Self::Freezing
        )
    }

    /// Returns what the one who comes down with the ailment is told.
    pub(crate) fn onset(self) -> &'static str {
        match self {
            Self::Hungry => "You are getting hungry.",
            Self::Starving => "You are starving. You must eat.",
            Self::Thirsty => "You are getting thirsty.",
            Self::Parched => "You are parched. You must drink.",
            Self::Tired => "You are getting tired.",
            Self::Exhausted => "You are exhausted and can barely stay on your feet.",
            Self::Chilled => "You are cold.",
            Self::Hypothermic => "You are shivering hard. You must get warm.",
            Self::Freezing => "You no longer feel the cold. You are freezing to death.",
            Self::Sick => "Your stomach cramps. You are sick.",
        }
    }
}

/// What someone died of.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Cause {
    Starvation,
    Thirst,
    Cold,
    Exhaustion,
    Sickness,
    OldAge,
}

impl Cause {
    pub fn name(self) -> &'static str {
        match self {
            Self::Starvation => "starvation",
            Self::Thirst => "thirst",
            Self::Cold => "the cold",
            Self::Exhaustion => "exhaustion",
            Self::Sickness => "sickness",
            Self::OldAge => "old age",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Body {
    /// Calories of food energy.
    energy: u32,
    /// Tenths of a milliliter of water to spare.
    water: u32,
    /// Seconds of sleep owed.
    fatigue: u32,
    /// Millionths of a degree.
    core: i32,
    /// Millionths of full health.
    health: u32,
    #[serde(default)]
    sick_until: Option<Timestamp>,
}

/// A rested body in good health that has eaten and drunk lately.
impl Default for Body {
    fn default() -> Self {
        Self {
            energy: ENERGY_FULL / 5 * 4,
            water: WATER_FULL / 5 * 4,
            fatigue: 0,
            core: CORE_NORMAL,
            health: HEALTH_FULL,
            sick_until: None,
        }
    }
}

impl Body {
    /// Returns how full of food the body is, in hundredths.
    pub fn energy_percent(&self) -> u32 {
        percent(self.energy, ENERGY_FULL)
    }

    /// Returns how much of its fill of water the body has, in hundredths.
    pub fn water_percent(&self) -> u32 {
        percent(self.water, WATER_FULL)
    }

    /// Returns how rested the body is, in hundredths.
    pub fn rest_percent(&self) -> u32 {
        100 - percent(self.fatigue.min(FATIGUE_LIMIT), FATIGUE_LIMIT)
    }

    pub fn health_percent(&self) -> u32 {
        percent(self.health, HEALTH_FULL)
    }

    /// Returns how warm the body is inside.
    pub fn temperature(&self) -> Temperature {
        Temperature::from_tenths(self.core / 100_000)
    }

    /// Returns how warm the body is, in hundredths of the way from dying of cold to well.
    pub fn warmth_percent(&self) -> u32 {
        let above_dead = (self.core - CORE_DEAD).max(0) as u32;
        percent(above_dead, (CORE_NORMAL - CORE_DEAD) as u32).min(100)
    }

    pub fn is_alive(&self) -> bool {
        self.health > 0 && self.core > CORE_DEAD
    }

    pub fn is_sick(&self) -> bool {
        self.sick_until.is_some()
    }

    /// Returns whether the body has no room for more food.
    pub fn is_full(&self) -> bool {
        self.energy >= ENERGY_FULL / 100 * 97
    }

    /// Returns whether the body has no room for more water.
    pub fn is_quenched(&self) -> bool {
        self.water >= WATER_FULL / 100 * 97
    }

    pub fn is_rested(&self) -> bool {
        self.fatigue == 0
    }

    /// Returns whether the body owes too little sleep to fall asleep.
    pub fn is_wide_awake(&self) -> bool {
        self.fatigue < 3_600
    }

    /// Returns whether the body is too cold to sleep.
    pub fn is_too_cold_to_sleep(&self) -> bool {
        self.core < CORE_WAKING
    }

    pub fn must_sleep(&self) -> bool {
        self.fatigue >= FATIGUE_LIMIT
    }

    /// Returns everything that is wrong with the body, with only the gravest of each kind.
    pub fn ailments(&self) -> Vec<Ailment> {
        let mut ailments = Vec::new();

        if self.energy == 0 {
            ailments.push(Ailment::Starving);
        } else if self.energy < ENERGY_FULL / 100 * 35 {
            ailments.push(Ailment::Hungry);
        }
        if self.water == 0 {
            ailments.push(Ailment::Parched);
        } else if self.water < WATER_FULL / 100 * 35 {
            ailments.push(Ailment::Thirsty);
        }
        if self.fatigue >= EXHAUSTED {
            ailments.push(Ailment::Exhausted);
        } else if self.fatigue >= TIRED {
            ailments.push(Ailment::Tired);
        }
        if self.core < CORE_FREEZING {
            ailments.push(Ailment::Freezing);
        } else if self.core < CORE_HYPOTHERMIC {
            ailments.push(Ailment::Hypothermic);
        } else if self.core < CORE_CHILLED {
            ailments.push(Ailment::Chilled);
        }
        if self.is_sick() {
            ailments.push(Ailment::Sick);
        }

        ailments
    }

    /// Returns how fast the body gets things done, in hundredths of how fast it does when
    /// nothing is wrong with it.
    pub fn pace_percent(&self) -> u32 {
        self.ailments()
            .into_iter()
            .map(|ailment| match ailment {
                Ailment::Hungry | Ailment::Thirsty | Ailment::Chilled => 100,
                Ailment::Tired => 90,
                Ailment::Sick => 80,
                Ailment::Starving | Ailment::Parched | Ailment::Hypothermic => 70,
                Ailment::Exhausted => 60,
                Ailment::Freezing => 40,
            })
            .min()
            .unwrap_or(100)
    }

    /// Takes in what one helping of `food` gives.
    pub(crate) fn take_in(&mut self, food: Food) {
        self.energy = (self.energy + food.energy * 1_000).min(ENERGY_FULL);
        self.drink(food.water);
    }

    /// Takes in `milliliters` of water.
    pub(crate) fn drink(&mut self, milliliters: u32) {
        self.water = (self.water + milliliters * 10).min(WATER_FULL);
    }

    /// Returns how many milliliters of water the body has room for.
    pub(crate) fn thirst(&self) -> u32 {
        (WATER_FULL - self.water) / 10
    }

    /// Makes the body sick until `until`, or until its present sickness ends if that is
    /// later.
    pub(crate) fn sicken(&mut self, until: Timestamp) {
        self.sick_until = self.sick_until.max(Some(until));
    }

    /// Cools the body by `millidegrees` thousandths of a degree.
    pub(crate) fn cool(&mut self, millidegrees: i32) {
        self.core -= millidegrees * 1_000;
    }

    /// Lets a minute pass that ends at `now`, spent at `effort` where it feels to be `felt`
    /// warm, on a `bed` or not. Returns what the body died of in that minute, if it did.
    pub(crate) fn pass_minute(
        &mut self,
        effort: Effort,
        felt: Temperature,
        bed: bool,
        now: Timestamp,
    ) -> Option<Cause> {
        if self.sick_until.is_some_and(|until| until <= now) {
            self.sick_until = None;
        }
        let sick = self.is_sick();
        let short_of_comfort = COMFORT.as_tenths() - (felt.as_tenths() + effort.warmth());
        let cold = short_of_comfort > 0;
        let losing_heat = short_of_comfort > SHIVER_RANGE;

        let mut energy = effort.energy();
        let mut water = effort.water();
        if cold {
            energy += energy * short_of_comfort.min(SHIVER_RANGE) as u32 / 100;
        }
        if sick {
            energy += energy / 4;
            water *= 2;
        }
        self.energy = self.energy.saturating_sub(energy);
        self.water = self.water.saturating_sub(water);

        if effort == Effort::Sleeping {
            let recovered = match (losing_heat, bed) {
                (true, _) => SLEEP.0,
                (false, false) => SLEEP.1,
                (false, true) => SLEEP.2,
            };
            self.fatigue = self.fatigue.saturating_sub(recovered);
        } else {
            self.fatigue = (self.fatigue + effort.fatigue()).min(FATIGUE_LIMIT);
        }

        if losing_heat {
            self.core -= (short_of_comfort - SHIVER_RANGE) * COOLING;
        } else {
            let warming = (12_000 + (SHIVER_RANGE - short_of_comfort) * 300).min(60_000);
            self.core = (self.core + warming).min(CORE_NORMAL);
        }

        let harms = [
            (Cause::Cold, FREEZING_HARM, self.core < CORE_FREEZING),
            (Cause::Thirst, PARCHED_HARM, self.water == 0),
            (Cause::Exhaustion, EXHAUSTED_HARM, self.fatigue >= EXHAUSTED),
            (Cause::Starvation, STARVING_HARM, self.energy == 0),
            (Cause::Sickness, SICK_HARM, sick),
        ];
        let harm: u32 = harms
            .iter()
            .filter(|(_, _, harmed)| *harmed)
            .map(|(_, harm, _)| harm)
            .sum();

        if harm > 0 {
            self.health = self.health.saturating_sub(harm);
        } else if self.energy > ENERGY_FULL / 5 && self.water > WATER_FULL / 5 {
            let healing = if effort == Effort::Sleeping {
                HEALING.1
            } else {
                HEALING.0
            };
            self.health = (self.health + healing).min(HEALTH_FULL);
        }

        if self.core <= CORE_DEAD {
            self.health = 0;
            return Some(Cause::Cold);
        }
        if self.health == 0 {
            return harms
                .iter()
                .find(|(_, _, harmed)| *harmed)
                .map(|(cause, _, _)| *cause);
        }
        None
    }

    /// Ends the life of the body.
    pub(crate) fn die(&mut self) {
        self.health = 0;
    }
}

fn percent(part: u32, whole: u32) -> u32 {
    (u64::from(part) * 100).div_ceil(u64::from(whole)) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    const MILD: Temperature = Temperature::from_degrees(20);
    const MINUTES_PER_DAY: u64 = 24 * 60;

    fn minute(number: u64) -> Timestamp {
        Timestamp::from_seconds(number * 60)
    }

    /// Lets the body live through days of resting and sleeping nine hours in the mild,
    /// with `care` taken of it before every minute, until it dies or `days` have passed.
    /// Returns what it died of and after how many days.
    fn live(body: &mut Body, days: u64, mut care: impl FnMut(&mut Body)) -> Option<(Cause, u64)> {
        for number in 0..days * MINUTES_PER_DAY {
            let effort = if number % MINUTES_PER_DAY < 9 * 60 {
                Effort::Sleeping
            } else {
                Effort::Resting
            };

            care(body);
            if let Some(cause) = body.pass_minute(effort, MILD, false, minute(number + 1)) {
                return Some((cause, number / MINUTES_PER_DAY));
            }
        }
        None
    }

    fn fed(body: &mut Body) {
        body.energy = ENERGY_FULL;
    }

    fn watered(body: &mut Body) {
        body.water = WATER_FULL;
    }

    #[test]
    fn body_that_lacks_nothing_stays_well() {
        let mut body = Body::default();

        let death = live(&mut body, 30, |body| {
            fed(body);
            watered(body);
        });

        assert_eq!(death, None);
        assert_eq!(body.health_percent(), 100);
        assert_eq!(body.ailments(), []);
        assert_eq!(body.pace_percent(), 100);
    }

    #[test]
    fn day_of_rest_burns_about_two_thousand_kilocalories() {
        let mut body = Body::default();
        fed(&mut body);

        live(&mut body, 1, watered);

        let burned = (ENERGY_FULL - body.energy) / 1_000;
        assert!((1_700..2_300).contains(&burned), "{burned} kcal");
    }

    #[test]
    fn hunger_kills_in_weeks_and_thirst_in_days() {
        let mut starved = Body::default();
        let mut parched = Body::default();

        let starvation = live(&mut starved, 100, watered);
        let thirst = live(&mut parched, 100, fed);

        let (cause, days) = starvation.expect("nobody lives a hundred days without food");
        assert_eq!(cause, Cause::Starvation);
        assert!((20..40).contains(&days), "starved in {days} days");
        assert!(!starved.is_alive());

        let (cause, days) = thirst.expect("nobody lives a hundred days without water");
        assert_eq!(cause, Cause::Thirst);
        assert!((3..7).contains(&days), "died of thirst in {days} days");
    }

    #[test]
    fn hunger_and_thirst_come_on_before_they_harm() {
        let mut body = Body::default();
        let mut seen = Vec::new();

        live(&mut body, 3, |body| {
            for ailment in body.ailments() {
                if !seen.contains(&ailment) {
                    seen.push(ailment);
                }
            }
        });

        assert_eq!(
            seen,
            [
                Ailment::Thirsty,
                Ailment::Hungry,
                Ailment::Parched,
                Ailment::Starving
            ]
        );
    }

    #[test]
    fn deep_frost_kills_within_hours_and_mild_cold_only_costs_energy() {
        let pass = |felt: Temperature, minutes: u64| {
            let mut body = Body::default();
            let death = (0..minutes).find_map(|number| {
                body.pass_minute(Effort::Resting, felt, false, minute(number + 1))
                    .map(|cause| (cause, number / 60))
            });
            (body, death)
        };

        let (_, death) = pass(Temperature::from_degrees(-17), 24 * 60);
        let (cause, hours) = death.expect("nobody rests a day in the deep frost");
        assert_eq!(cause, Cause::Cold);
        assert!((4..10).contains(&hours), "froze in {hours} hours");

        let (cool, death) = pass(Temperature::from_degrees(14), 8 * 60);
        let (warm, _) = pass(MILD, 8 * 60);
        assert_eq!(death, None);
        assert_eq!(cool.temperature(), Temperature::from_degrees(37));
        assert!(cool.energy < warm.energy);
    }

    #[test]
    fn cold_body_warms_up_again_where_it_is_warm() {
        let mut body = Body::default();
        for number in 0..240 {
            body.pass_minute(
                Effort::Resting,
                Temperature::from_degrees(-5),
                false,
                minute(number),
            );
        }
        assert!(body.ailments().contains(&Ailment::Hypothermic));
        assert!(body.is_too_cold_to_sleep());
        assert_eq!(body.pace_percent(), 70);

        for number in 240..480 {
            body.pass_minute(
                Effort::Resting,
                Temperature::from_degrees(30),
                false,
                minute(number),
            );
        }

        assert_eq!(body.temperature(), Temperature::from_degrees(37));
        assert_eq!(body.warmth_percent(), 100);
    }

    #[test]
    fn effort_keeps_a_body_warm_where_rest_does_not() {
        let cool = |effort| {
            let mut body = Body::default();
            for number in 0..120 {
                body.pass_minute(effort, Temperature::from_degrees(8), false, minute(number));
            }
            body.core
        };

        assert!(cool(Effort::Sleeping) < cool(Effort::Resting));
        assert!(cool(Effort::Resting) < CORE_NORMAL);
        assert_eq!(cool(Effort::Working), CORE_NORMAL);
    }

    #[test]
    fn sleep_pays_back_a_day_awake_in_a_night() {
        let mut body = Body::default();
        for number in 0..16 * 60 {
            fed(&mut body);
            watered(&mut body);
            body.pass_minute(Effort::Resting, MILD, false, minute(number));
        }
        assert!(!body.is_rested());
        assert!(!body.is_wide_awake());
        let owed = body.fatigue;

        let mut on_a_bed = body.clone();
        for number in 0..450 {
            body.pass_minute(Effort::Sleeping, MILD, false, minute(number));
            on_a_bed.pass_minute(Effort::Sleeping, MILD, true, minute(number));
        }
        assert!(!body.is_rested());
        assert!(on_a_bed.is_rested());

        for number in 0..90 {
            body.pass_minute(Effort::Sleeping, MILD, false, minute(number));
        }
        assert!(body.is_rested(), "{owed} seconds were owed");
    }

    #[test]
    fn going_without_sleep_wears_a_body_down_until_it_must_sleep() {
        let mut body = Body::default();
        let mut tired_after = None;

        for number in 0..5 * MINUTES_PER_DAY {
            fed(&mut body);
            watered(&mut body);
            body.pass_minute(Effort::Working, MILD, false, minute(number));
            if tired_after.is_none() && body.ailments().contains(&Ailment::Tired) {
                tired_after = Some(number / 60);
            }
            if body.must_sleep() {
                break;
            }
        }

        assert!((10..20).contains(&tired_after.expect("work tires")));
        assert!(body.must_sleep());
        assert!(body.ailments().contains(&Ailment::Exhausted));
        assert!(body.health_percent() < 100);
        assert_eq!(body.rest_percent(), 0);
    }

    #[test]
    fn sickness_passes_and_health_comes_back_after_it() {
        let mut body = Body::default();
        body.sicken(minute(600));
        body.sicken(minute(300));

        for number in 0..600 {
            fed(&mut body);
            watered(&mut body);
            body.pass_minute(Effort::Resting, MILD, false, minute(number));
        }
        assert!(body.is_sick());
        let worst = body.health;
        assert!(worst < HEALTH_FULL);

        for number in 600..700 {
            fed(&mut body);
            watered(&mut body);
            body.pass_minute(Effort::Resting, MILD, false, minute(number));
        }
        assert!(!body.is_sick());
        assert!(body.health > worst);
    }

    #[test]
    fn eating_and_drinking_fill_a_body_no_fuller_than_full() {
        let mut body = Body {
            energy: 0,
            water: 0,
            ..Body::default()
        };
        let food = Food {
            energy: 450,
            water: 100,
            risk: 0,
            seconds: 60,
        };

        body.take_in(food);
        assert_eq!(body.energy, 450_000);
        assert_eq!(body.thirst(), 2_900);
        assert!(!body.is_full());

        for _ in 0..10 {
            body.take_in(food);
        }
        body.drink(5_000);
        assert!(body.is_full() && body.is_quenched());
        assert_eq!(body.energy_percent(), 100);
        assert_eq!(body.water_percent(), 100);
        assert_eq!(body.thirst(), 0);
    }

    #[test]
    fn body_reads_back_as_it_was_written() {
        let mut body = Body::default();
        body.sicken(minute(5));
        body.cool(1_500);

        let written = ron::to_string(&body).expect("the body encodes");
        let read: Body = ron::from_str(&written).expect("the body decodes");

        assert_eq!(read, body);
    }
}
