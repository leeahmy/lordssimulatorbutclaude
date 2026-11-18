//! Numbers that are the same for the same seed on every machine.

use serde::{Deserialize, Serialize};

/// The SplitMix64 generator, which gives the same numbers for the same seed everywhere.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Rng(u64);

impl Rng {
    pub const fn new(seed: u64) -> Self {
        Self(seed)
    }

    pub(crate) fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        mix(self.0)
    }

    /// Returns a number from zero up to but not including `bound`.
    pub(crate) fn below(&mut self, bound: u64) -> u64 {
        ((u128::from(self.next()) * u128::from(bound)) >> 64) as u64
    }

    /// Returns a number from `low` to `high` with both included.
    pub(crate) fn between(&mut self, (low, high): (i64, i64)) -> i64 {
        debug_assert!(low <= high);
        low + self.below((high - low + 1) as u64) as i64
    }

    /// Returns a number from `low` to `high` that is more often near `low`.
    pub(crate) fn mostly_low(&mut self, range: (i64, i64)) -> i64 {
        self.between(range).min(self.between(range))
    }

    /// Returns whether a chance of `percent` in a hundred came true. A chance below zero
    /// never does, and one above a hundred always does.
    pub(crate) fn chance(&mut self, percent: u64) -> bool {
        self.below(100) < percent
    }

    /// Returns whether a chance of `share` in `out_of` came true.
    pub(crate) fn one_in(&mut self, share: u64, out_of: u64) -> bool {
        self.below(out_of) < share
    }

    /// Returns a number from zero up to but not including one.
    pub(crate) fn unit(&mut self) -> f64 {
        unit(self.next())
    }
}

/// Returns a number from zero up to but not including one that is always the same for the
/// same `seed`, `channel`, and `index`.
pub(crate) fn fixed_unit(seed: u64, channel: u64, index: i64) -> f64 {
    let stirred = mix(seed ^ mix(channel.wrapping_mul(0xD6E8_FEB8_6659_FD93)));
    unit(mix(stirred.wrapping_add(
        (index as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15),
    )))
}

fn mix(mut mixed: u64) -> u64 {
    mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    mixed ^ (mixed >> 31)
}

fn unit(bits: u64) -> f64 {
    (bits >> 11) as f64 / (1_u64 << 53) as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_stay_within_their_bounds() {
        let mut rng = Rng::new(42);

        for index in 0..10_000 {
            assert!(rng.below(7) < 7);
            assert!((-3..=3).contains(&rng.between((-3, 3))));
            assert!((0.0..1.0).contains(&rng.unit()));
            assert!((0.0..1.0).contains(&fixed_unit(42, 3, index)));
        }
    }

    #[test]
    fn certain_and_impossible_chances_are_just_that() {
        let mut rng = Rng::new(7);

        for _ in 0..1_000 {
            assert!(rng.chance(100));
            assert!(!rng.chance(0));
            assert!(rng.one_in(5, 5));
            assert!(!rng.one_in(0, 5));
        }
    }

    #[test]
    fn fixed_numbers_depend_on_everything_they_are_made_of() {
        let number = fixed_unit(1, 2, 3);

        assert_eq!(number, fixed_unit(1, 2, 3));
        assert_ne!(number, fixed_unit(2, 2, 3));
        assert_ne!(number, fixed_unit(1, 3, 3));
        assert_ne!(number, fixed_unit(1, 2, 4));
    }
}
