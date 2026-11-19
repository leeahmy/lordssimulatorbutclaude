//! What the player character does with their own coin, who their kin are, and the
//! chronicle a settlement keeps of itself.

use crate::journal::Tone;
use crate::object::Living;
use crate::people::{Person, PersonId};
use crate::settlement::{Chronicled, building};
use crate::view::KinView;
use crate::world::{Kind, StructureKind};

use super::Simulation;

/// The most a chronicle holds. The oldest of what happens every year makes way first.
const LONGEST_CHRONICLE: usize = 400;

/// The least a share in a workplace costs, and the hundredths of what those who work
/// there earn that come to whoever owns it.
const LEAST_SHARE: i64 = 200;
const OWNERS_CUT: i64 = 25;

/// How many coins given to the treasury it takes for people to think a point better of
/// the giver, and how many points one gift is good for at the most.
const COINS_PER_THANKS: u32 = 50;
const MOST_THANKS: u32 = 5;

/// How many tenths of a thinker's day a coin given to those who think buys.
const INSIGHT_PER_COIN: u32 = 2;

impl Simulation {
    /// Writes in the chronicle of the settlement, which is kept for good.
    pub(super) fn chronicle(&mut self, text: impl Into<String>) {
        let time = self.state.time;
        if let Some(settlement) = &mut self.state.settlement {
            if settlement.chronicle.len() >= LONGEST_CHRONICLE {
                settlement.chronicle.remove(LONGEST_CHRONICLE / 4);
            }
            settlement.chronicle.push(Chronicled {
                time,
                text: text.into(),
            });
            self.society_stale = true;
        }
    }

    /// Returns what a share in a workplace of `kind` costs.
    pub(super) fn share_price(kind: StructureKind) -> Option<i64> {
        let plan = building(kind)?;
        plan.jobs?;
        Some(i64::from(plan.coins).max(LEAST_SHARE))
    }

    /// Returns how many shares the player character holds in workplaces of `kind`.
    fn shares(&self, kind: StructureKind) -> u32 {
        let held = self.state.holdings.iter().find(|(held, _)| *held == kind);
        held.map_or(0, |(_, shares)| *shares)
    }

    /// Buys the player character a share in a workplace of `kind`, one for each that
    /// stands at the most.
    pub(super) fn invest(&mut self, kind: StructureKind) {
        let name = Kind::Structure(kind).name().to_lowercase();
        let standing = self.built().iter().filter(|built| **built == kind).count() as u32;
        let refusal = match Self::share_price(kind) {
            _ if !self.monetized() => Some("Nothing is bought or sold without a market.".into()),
            None => Some(format!("There is no owning a {name}.")),
            Some(_) if standing <= self.shares(kind) => {
                Some(format!("There is no {name} left to buy into."))
            }
            Some(price) if price > self.state.purse => Some(format!(
                "A share in a {name} costs {price} coins, and you have {}.",
                self.state.purse.max(0)
            )),
            Some(_) => None,
        };
        if let Some(refusal) = refusal {
            self.note(Tone::Plain, refusal);
            return;
        }

        let price = Self::share_price(kind).unwrap_or(LEAST_SHARE);
        self.charge(price);
        match self
            .state
            .holdings
            .iter_mut()
            .find(|(held, _)| *held == kind)
        {
            Some((_, shares)) => *shares += 1,
            None => self.state.holdings.push((kind, 1)),
        }
        self.note(
            Tone::Good,
            format!(
                "You buy into a {name} for {price} coins. A quarter of what it earns is yours."
            ),
        );
    }

    /// Pays the player character what their shares earned in a day, less the taxes on
    /// it, which go to the treasury. Returns what was paid.
    pub(super) fn pay_dividends(&mut self, working: impl Fn(crate::people::Role) -> u32) -> i64 {
        let kinds = self.built();
        let tax = i64::from(self.tax_rate());
        let mut earned = 0;
        // What has burned down or been pulled down earns nothing, and is no longer held.
        self.state.holdings.retain_mut(|(kind, shares)| {
            let standing = kinds.iter().filter(|built| *built == kind).count() as u32;
            *shares = (*shares).min(standing);
            *shares > 0
        });
        for (kind, shares) in &self.state.holdings {
            let Some((role, places)) = building(*kind).and_then(|plan| plan.jobs) else {
                continue;
            };
            let standing = kinds.iter().filter(|built| *built == kind).count() as u32;
            // Each of them is as well manned as all of them are between them.
            let manned = working(role).min(places * standing);
            let wages = i64::from(manned) * i64::from(role.wage()) / i64::from(standing.max(1));
            earned += wages * i64::from(*shares) * OWNERS_CUT / 100;
        }

        let taxed = earned * tax / 100;
        self.state.purse += earned - taxed;
        self.state.income = earned - taxed;
        if let Some(settlement) = &mut self.state.settlement {
            settlement.treasury += taxed;
        }
        earned - taxed
    }

    /// Gives `coins` out of the purse of the player character to the treasury, for which
    /// people think better of them.
    pub(super) fn donate(&mut self, coins: u32) {
        let coins = i64::from(coins).min(self.state.purse.max(0));
        if coins <= 0 || self.state.settlement.is_none() {
            self.note(Tone::Plain, "You have nothing to give.");
            return;
        }
        self.state.purse -= coins;
        if let Some(settlement) = &mut self.state.settlement {
            settlement.treasury += coins;
        }
        let thanks = (coins as u32 / COINS_PER_THANKS).min(MOST_THANKS) as u8;
        for id in self.members() {
            if let Some(person) = self.state.world.person_mut(id) {
                person.warm_to(thanks);
            }
        }
        self.society_stale = true;
        self.note(
            Tone::Good,
            format!("You give {coins} coins to the treasury, and it is not forgotten."),
        );
        if coins >= 500 {
            let name = self.state.player.name().full();
            self.chronicle(format!("{name} gave {coins} coins to the treasury."));
        }
    }

    /// Gives `coins` out of the purse of the player character to those who think, which
    /// brings what is being worked out on.
    pub(super) fn endow(&mut self, coins: u32) {
        let coins = i64::from(coins).min(self.state.purse.max(0));
        let pursuing = self
            .state
            .settlement
            .as_ref()
            .is_some_and(|settlement| settlement.pursuit.is_some());
        if coins <= 0 {
            self.note(Tone::Plain, "You have nothing to give.");
            return;
        }
        if !pursuing {
            self.note(Tone::Plain, "Nothing is being worked out to give towards.");
            return;
        }
        self.state.purse -= coins;
        self.note(
            Tone::Good,
            format!("You give {coins} coins to those who think, to keep them at it."),
        );
        self.work_out(coins as u32 * INSIGHT_PER_COIN);
        self.society_stale = true;
    }

    /// Returns the kin of the player character, dead or alive: their parents, their
    /// partner, their brothers and sisters, their children, and their grandchildren.
    pub(super) fn kin(&self) -> Vec<KinView> {
        let world = &self.state.world;
        let now = self.state.time;
        let Some(me) = world.protagonist() else {
            return Vec::new();
        };
        let Some(mine) = world.person(me) else {
            return Vec::new();
        };
        let child_of = |person: &Person, parent: PersonId| person.parents.contains(&Some(parent));
        let children: Vec<PersonId> = world
            .everyone()
            .filter(|(_, person)| child_of(person, me))
            .map(|(id, _)| id)
            .collect();

        let mut kin = Vec::new();
        for (id, person) in world.everyone().filter(|(id, _)| *id != me) {
            let relation = if mine.parents.contains(&Some(id)) {
                "Parent"
            } else if mine.partner == Some(id) {
                "Partner"
            } else if child_of(person, me) {
                "Child"
            } else if mine
                .parents
                .iter()
                .flatten()
                .any(|parent| child_of(person, *parent))
            {
                "Brother or sister"
            } else if children.iter().any(|child| child_of(person, *child)) {
                "Grandchild"
            } else {
                continue;
            };
            kin.push(KinView {
                name: person.name.full(),
                relation,
                age: person.died.map_or(person.age(now), |died| person.age(died)),
                alive: person.died.is_none(),
                here: person.is_present() && person.member,
            });
        }
        kin
    }
}

#[cfg(test)]
mod tests;
