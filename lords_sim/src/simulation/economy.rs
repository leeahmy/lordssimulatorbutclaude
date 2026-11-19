//! What a settlement makes of its days: the work of every trade, the harvest, what is
//! worked out, and, once there is coin, what is taxed, paid, bought, and sold.

use crate::item::{Inventory, Item, Stack};
use crate::journal::Tone;
use crate::people::{PersonId, Role, Trait};
use crate::politics::{Law, Office};
use crate::settlement::{Advance, HIGHEST_TAX, Ledger, Running, WORKDAY, building, places};
use crate::time::gregorian::Date;
use crate::work::{PROCESSES, Process, ceiling};
use crate::world::{PlantSpecies, StructureKind, ThingId};

use super::Simulation;

/// The days of the year between which rye grows, and the day it is harvested.
pub(super) const GROWING_SEASON: (u16, u16) = (121, 258);
const HARVEST_DAY: u16 = 259;
const GRAIN_PER_FARMER_DAY: u32 = 30;

/// How many pelts a set of winter clothes takes.
const PELTS_PER_OUTFIT: u32 = 4;

/// How many fishers, trappers, and foragers the lake and the forest feed before each
/// further one brings in a quarter of what the others do.
pub(super) const FISHING_GROUNDS: u32 = 10;
pub(super) const TRAPPING_GROUNDS: u32 = 6;
pub(super) const FORAGING_GROUNDS: u32 = 8;
pub(super) const HUNTING_GROUNDS: u32 = 4;

/// How many days of work by everyone who works it takes to wear out one tool.
const TOOL_LIFE: u32 = 80;

/// How many coins' worth a merchant, a sailor, and a transport worker trade in a day.
const MERCHANT_TRADE: u32 = 30;
const SAILOR_TRADE: u32 = 60;
const CARRIER_TRADE: u32 = 400;

/// The hundredths of what something is worth that are had for it when it is sold outside,
/// and that are paid for it when it is bought there.
const EXPORT_PRICE: u32 = 60;
const IMPORT_PRICE: u32 = 130;

/// The hundredths of what something is worth that the player character pays for it at
/// the market, and is paid for it there.
const BUYING_PRICE: i64 = 120;
const SELLING_PRICE: i64 = 80;

/// How many days of food a settlement keeps before it sells any, and below which it buys.
const FOOD_TO_SPARE: u32 = 240;
const FOOD_TO_BUY: u32 = 20;

/// How many people one healer and one teacher see to.
pub(super) const PEOPLE_PER_HEALER: u32 = 60;
const PEOPLE_PER_TEACHER: u32 = 100;

/// How many days pass between each point by which reading spreads or is lost.
const LITERACY_DAYS: u16 = 40;

/// What whoever leads is paid a day out of the treasury.
const STIPEND: i64 = 10;

/// What the settlement has by way of who works at what, after everyone who has no place
/// to work at has been left out.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Hands {
    by_role: Vec<u32>,
    /// How good those of each trade are at it, added up, and how many they are before
    /// those with no place to work at are left out.
    skill: Vec<u32>,
    all: Vec<u32>,
    /// How many of those who think, study, and teach are clever, and how many of the
    /// clerks are stern.
    pub clever: u32,
    pub stern: u32,
    /// Everyone old enough to work.
    pub workers: u32,
    /// Those hired by the player character to think.
    pub hired: u32,
}

impl Hands {
    pub fn of(&self, role: Role) -> u32 {
        self.by_role.get(role as usize).copied().unwrap_or(0)
    }

    /// Returns how many hundredths of what a beginner does those of `role` do, by how
    /// good they are at it: three tenths more for masters all.
    pub fn mastery(&self, role: Role) -> u32 {
        let all = self.all.get(role as usize).copied().unwrap_or(0);
        let skill = self.skill.get(role as usize).copied().unwrap_or(0);
        100 + skill / all.max(1) * 3 / 10
    }
}

impl Simulation {
    /// Returns whether goods are bought and sold for coin in the settlement.
    pub(super) fn monetized(&self) -> bool {
        let settlement = self.state.settlement.as_ref();
        settlement.is_some_and(|settlement| settlement.knows(Advance::Coinage))
            && self.has_built(StructureKind::Market)
    }

    pub(super) fn knows_advance(&self, advance: Advance) -> bool {
        let settlement = self.state.settlement.as_ref();
        settlement.is_some_and(|settlement| settlement.knows(advance))
    }

    /// Returns the kinds of everything that stands finished.
    pub(super) fn built(&self) -> Vec<StructureKind> {
        self.structures()
            .into_iter()
            .map(|(_, kind)| kind)
            .filter(|kind| *kind != StructureKind::Site)
            .collect()
    }

    pub(super) fn has_built(&self, kind: StructureKind) -> bool {
        self.kept.iter().any(|id| {
            let structure = self.state.world.structure(*id);
            structure.is_some_and(|structure| structure.kind() == kind)
        })
    }

    /// Returns who of `members` works at what, those without a place to work at left out.
    pub(super) fn hands(&self, members: &[PersonId], kinds: &[StructureKind]) -> Hands {
        let now = self.state.time;
        let working_age = self.working_age();
        let mut hands = Hands {
            by_role: vec![0; Role::ALL.len()],
            skill: vec![0; Role::ALL.len()],
            all: vec![0; Role::ALL.len()],
            ..Hands::default()
        };

        for id in members {
            let Some(person) = self.state.world.person(*id) else {
                continue;
            };
            if person.age(now) < working_age {
                continue;
            }
            hands.workers += 1;
            // Someone hired by the player thinks, whatever else they are set to.
            if person.hired_until.is_some_and(|until| until > now) {
                hands.hired += 1;
            } else {
                let role = person.role;
                hands.by_role[role as usize] += 1;
                hands.all[role as usize] += 1;
                hands.skill[role as usize] += u32::from(person.skill);
                let minds = matches!(role, Role::Thinker | Role::Scholar | Role::Teacher);
                if minds && person.has(Trait::Clever) {
                    hands.clever += 1;
                }
                if role == Role::Clerk && person.has(Trait::Stern) {
                    hands.stern += 1;
                }
            }
        }
        for role in Role::ALL {
            if let Some(places) = places(role, kinds) {
                let count = &mut hands.by_role[role as usize];
                *count = (*count).min(places);
            }
        }
        hands
    }

    /// Returns how many hundredths of what the trade yields by itself it yields with
    /// what the settlement knows and has.
    fn yield_percent(&self, role: Role, kinds: &[StructureKind], tools: u32) -> u32 {
        let Some(settlement) = &self.state.settlement else {
            return 100;
        };
        let knows = |advance| settlement.knows(advance);
        let mut percent = tools;

        if knows(Advance::Wheel)
            && matches!(role, Role::Woodcutter | Role::Gatherer | Role::Quarrier)
        {
            percent = percent * 3 / 2;
        }
        if role.is_craft() {
            if kinds.contains(&StructureKind::Guildhall) {
                percent = percent * 5 / 4;
            }
            if settlement.lives_by(Law::GuildRights) {
                percent = percent * 11 / 10;
            }
        }
        if knows(Advance::SteamPower) && matches!(role, Role::Miner | Role::Sawyer) {
            percent *= 2;
        }
        let powered = matches!(
            role,
            Role::FactoryHand | Role::MillHand | Role::Steelworker | Role::Canner | Role::Printer
        );
        if powered && settlement.running.power {
            percent = percent * 3 / 2;
        }
        if role == Role::FactoryHand && knows(Advance::Electronics) {
            percent = percent * 3 / 2;
        }
        percent
    }

    /// Returns how many hundredths of what a farmer grows by themselves they grow with
    /// what the settlement knows and has.
    pub(super) fn farm_percent(&self, tools: u32) -> u32 {
        let Some(settlement) = &self.state.settlement else {
            return 100;
        };
        let mut percent = tools;

        if settlement.knows(Advance::Plough) {
            percent = percent * 5 / 4;
        }
        if settlement.knows(Advance::CropRotation) {
            percent = percent * 6 / 5;
        }
        if settlement.knows(Advance::Agronomy) {
            percent *= 2;
        }
        if settlement.running.tractors {
            percent *= 3;
        }
        percent
    }

    /// Returns how many hundredths of what it yields without, work yields with the tools
    /// there are in the store.
    pub(super) fn tools_percent(&self) -> u32 {
        let stocked = self.store().is_some_and(|store| store.has(Item::Tools));
        match &self.state.settlement {
            Some(settlement) if stocked => settlement.tool_percent(),
            _ => 100,
        }
    }

    /// Has `workers` of `role` do a day of what their trade does, out of and into
    /// `store`, at `percent` hundredths of what it yields by itself. Returns how many of
    /// them had what their work takes.
    fn ply(
        &self,
        role: Role,
        workers: u32,
        percent: u32,
        people: u32,
        store: &mut Inventory,
    ) -> u32 {
        let processes: Vec<&Process> = PROCESSES
            .iter()
            .filter(|process| process.role == role)
            .filter(|process| {
                process
                    .needs
                    .is_none_or(|needed| self.knows_advance(needed))
            })
            .collect();
        if processes.is_empty() {
            return workers;
        }

        let smoked = self.has_built(StructureKind::Smokehouse);
        let runnable = |process: &Process, store: &Inventory| {
            let supplied = process
                .takes
                .iter()
                .all(|(item, count)| store.count(*item) >= *count);
            let wanted = process
                .makes
                .first()
                .is_none_or(|(item, _)| store.count(*item) < ceiling(*item, people));
            supplied && wanted
        };

        let mut runs = vec![0u32; processes.len()];
        let mut active = 0;
        let no_main = processes.iter().all(|process| process.aside);
        for _ in 0..workers {
            let main = processes
                .iter()
                .position(|process| !process.aside && runnable(process, store));
            if let Some(index) = main {
                for (item, count) in processes[index].takes {
                    store.take(*item, *count);
                }
                runs[index] += 1;
                active += 1;
            } else if no_main {
                active += 1;
            }
            for (index, process) in processes.iter().enumerate() {
                if process.aside && runnable(process, store) {
                    for (item, count) in process.takes {
                        store.take(*item, *count);
                    }
                    runs[index] += 1;
                }
            }
        }

        for (process, runs) in processes.iter().zip(runs) {
            for (item, count) in process.makes {
                // What is caught is smoked where there is a smokehouse, and keeps.
                let item = match item {
                    Item::CookedFish if smoked => Item::DriedFish,
                    Item::CookedMeat if smoked => Item::DriedMeat,
                    item => *item,
                };
                let made = count * runs * percent / 100;
                let room = ceiling(item, people).saturating_sub(store.count(item));
                store.add(item, made.min(room));
            }
        }
        active
    }

    /// Has those who work do a day of it.
    pub(super) fn work_day(&mut self, members: &[PersonId]) {
        let now = self.state.time;
        let date = Date::from_timestamp(now);
        let kinds = self.built();
        let count = |kind| kinds.iter().filter(|built| **built == kind).count() as u32;
        let Some(settlement) = &self.state.settlement else {
            return;
        };
        let knows = |advance| settlement.knows(advance);
        let lives_by = |law| settlement.lives_by(law);
        let people = members.len() as u32 + 1;
        let hands = self.hands(members, &kinds);
        let tools = self.tools_percent();
        let food_share = if lives_by(Law::OwnHearths) {
            115
        } else if lives_by(Law::SharedTable) {
            95
        } else {
            100
        };
        let unpaid = settlement.ledger.unpaid;
        let mut store = self.store().cloned().unwrap_or_default();
        let mut running = Running {
            unheated: settlement.running.unheated,
            ..Running::default()
        };

        // What follows the seasons and what the lake and the forest give only so much of.
        let thinned =
            |workers: u32, grounds: u32| workers.min(grounds) * 4 + workers.saturating_sub(grounds);
        let foragers = thinned(hands.of(Role::Forager), FORAGING_GROUNDS);
        let (berries, each) = if PlantSpecies::Bilberry.is_ripe(date) {
            (Item::Bilberries, 160)
        } else if PlantSpecies::Lingonberry.is_ripe(date) {
            (Item::Lingonberries, 180)
        } else {
            (Item::InnerBark, 40)
        };
        store.add(
            berries,
            each * foragers / 4 * tools / 100 * food_share / 100,
        );

        let mut grounds = FISHING_GROUNDS;
        if count(StructureKind::Dock) > 0 {
            grounds += FISHING_GROUNDS;
        }
        if count(StructureKind::Harbour) > 0 {
            grounds += 2 * FISHING_GROUNDS;
        }
        let boats = if count(StructureKind::Dock) > 0 { 2 } else { 1 };
        let ice = if self.conditions.frozen { 2 } else { 1 };
        let fishing = thinned(hands.of(Role::Fisher), grounds) * 25 * boats / ice;
        let trapping = thinned(hands.of(Role::Trapper), TRAPPING_GROUNDS) * 25;
        let hunting = thinned(hands.of(Role::Hunter), HUNTING_GROUNDS) * 25;

        for role in Role::ALL {
            let workers = hands.of(role);
            if workers == 0 {
                continue;
            }
            let percent = self.yield_percent(role, &kinds, tools) * hands.mastery(role) / 100;
            // Fishers and trappers are counted in hundredths of one working freely.
            let active = match role {
                Role::Fisher => self.ply(
                    role,
                    1,
                    percent * fishing / 100 * food_share / 100,
                    people,
                    &mut store,
                ),
                Role::Trapper => self.ply(
                    role,
                    1,
                    percent * trapping / 100 * food_share / 100,
                    people,
                    &mut store,
                ),
                Role::Hunter => self.ply(
                    role,
                    1,
                    percent * hunting / 100 * food_share / 100,
                    people,
                    &mut store,
                ),
                Role::Herder => self.ply(
                    role,
                    workers,
                    percent * food_share / 100,
                    people,
                    &mut store,
                ),
                Role::Forager
                | Role::Farmer
                | Role::Builder
                | Role::Crafter
                | Role::Thinker
                | Role::Idle => workers,
                _ => self.ply(role, workers, percent, people, &mut store),
            };
            // Those paid out of the treasury do half of what they would when it is empty.
            let active = if unpaid && role.is_public() {
                active.div_ceil(2)
            } else {
                active
            };
            match role {
                Role::Engineer => {
                    running.power = active > 0 && count(StructureKind::PowerStation) > 0;
                }
                Role::Mechanic => running.tractors = active > 0,
                Role::Innkeeper => running.tavern = active > 0,
                Role::Healer => running.healers = active,
                Role::Teacher => running.teachers = active,
                Role::Guard => running.guards = active,
                Role::Priest => running.priests = active,
                Role::Performer => running.performers = active,
                Role::Clerk => running.clerks = active + hands.stern.min(active),
                Role::Miller => running.millers = active,
                Role::Merchant => running.trade += active * MERCHANT_TRADE,
                Role::Sailor => {
                    let far = if knows(Advance::Navigation) { 2 } else { 1 };
                    running.trade += active * SAILOR_TRADE * far;
                }
                Role::Carrier => running.trade += active * CARRIER_TRADE,
                _ => {}
            }
        }
        let wired = count(StructureKind::TelegraphOffice) > 0 && hands.of(Role::Operator) > 0;
        if wired {
            running.trade = running.trade * 6 / 5;
        }
        if knows(Advance::Internet) {
            running.trade = running.trade * 6 / 5;
        }
        if lives_by(Law::FreeMarket) {
            running.trade = running.trade * 3 / 2;
        }
        if lives_by(Law::FreePort) {
            running.trade = running.trade * 3 / 2;
        }
        // What is burned and made fouls the air and the water, less so where there is
        // clean power, waterworks, and somewhere green.
        let smoke = 2 * count(StructureKind::Brickworks)
            + 3 * count(StructureKind::Steelworks)
            + 3 * count(StructureKind::Factory)
            + 2 * count(StructureKind::TextileMill)
            + count(StructureKind::CharcoalKiln)
            + if knows(Advance::CleanPower) {
                0
            } else {
                10 * count(StructureKind::PowerStation)
            };
        let cleaned =
            8 * count(StructureKind::Waterworks).min(1) + 5 * count(StructureKind::Park).min(1);
        running.pollution = smoke.saturating_sub(cleaned).min(100);
        let (pacts, ahead) = self.neighbourly();
        running.trade = running.trade * (6 + pacts) / 6;
        if !knows(Advance::Coinage) || count(StructureKind::Market) == 0 {
            running.trade = 0;
        }

        // The fields.
        let growing_season = (GROWING_SEASON.0..=GROWING_SEASON.1).contains(&date.ordinal());
        let grown = if growing_season {
            hands.of(Role::Farmer) * GRAIN_PER_FARMER_DAY * self.farm_percent(tools) / 100
                * hands.mastery(Role::Farmer)
                / 100
        } else {
            0
        };
        let rotated = knows(Advance::CropRotation);
        let potatoes = knows(Advance::Navigation);
        let milled = settlement.running.millers > 0 || running.millers > 0;

        // What is worked out. Insight is counted in tenths of a thinker's day. A little
        // comes to any band of people by itself, and more with every stranger traded with.
        // The clever count twice, near enough.
        let thinkers = hands.of(Role::Thinker) + hands.hired + hands.clever;
        let mut scholars = hands.of(Role::Scholar) * 30;
        if unpaid {
            scholars /= 2;
        }
        if count(StructureKind::ComputingCentre) > 0 {
            scholars *= 2;
        }
        if knows(Advance::Internet) {
            scholars = scholars * 3 / 2;
        }
        if lives_by(Law::FreeEnquiry) {
            scholars = scholars * 5 / 4;
        }
        let mut insight = thinkers * 10 + scholars;
        if count(StructureKind::Workshop) > 0 {
            insight = insight * 3 / 2;
        }
        if knows(Advance::Writing) {
            insight = insight * 3 / 2;
        }
        if knows(Advance::Arithmetic) {
            insight = insight * 11 / 10;
        }
        if knows(Advance::Optics) {
            insight = insight * 6 / 5;
        }
        insight = insight * (200 + u32::from(settlement.literacy)) / 200;
        if count(StructureKind::Museum) > 0 {
            insight = insight * 21 / 20;
        }
        if count(StructureKind::Library) > 0 {
            insight = insight * (400 + store.count(Item::Book).min(200)) / 400;
        }
        if lives_by(Law::GuildRights) {
            insight = insight * 9 / 10;
        }
        insight = insight * (20 + self.office(Office::Chancellor)) / 20;
        if members.len() >= 2 {
            insight += 3;
        }
        insight += running.trade / 10;
        // What neighbours who know more are seen to do is half learned already.
        insight = insight * (20 + ahead) / 20;
        insight = insight
            * if lives_by(Law::Schooling) {
                150
            } else if lives_by(Law::OldCustoms) {
                70
            } else {
                100
            }
            / 100;

        // Tools wear out with use.
        let mut wear = settlement.wear + hands.workers;
        while wear >= TOOL_LIFE {
            wear -= TOOL_LIFE;
            store.take(Item::Tools, 1);
        }

        // Reading spreads where there are teachers, and is lost where there are none.
        let taught = (running.teachers * PEOPLE_PER_TEACHER * 100 / people).min(95) as u8;
        let scribes = if knows(Advance::Writing) { 5 } else { 0 };
        let schooled = taught.max(scribes);
        let literacy = settlement.literacy;
        let literacy = if !date.ordinal().is_multiple_of(LITERACY_DAYS) {
            literacy
        } else if literacy < schooled {
            literacy + 1
        } else if literacy > schooled {
            literacy - 1
        } else {
            literacy
        };

        if let Some(kept) = self.store_mut() {
            *kept = store;
        }
        let harvest = self.state.settlement.as_mut().map_or(0, |settlement| {
            settlement.growing += grown;
            settlement.running = running;
            settlement.wear = wear;
            settlement.literacy = literacy;
            if date.ordinal() == HARVEST_DAY {
                std::mem::take(&mut settlement.growing)
            } else {
                0
            }
        });
        if harvest > 0 {
            let harvest = if milled { harvest * 11 / 10 } else { harvest };
            // One summer in ten is too dry, and one in ten is as good as they come.
            let harvest = match self.state.rng.below(10) {
                0 => {
                    self.alert = true;
                    self.note(Tone::Bad, "The summer was dry, and the harvest is poor.");
                    harvest * 6 / 10
                }
                1 => {
                    self.note(Tone::Good, "The summer was kind, and the harvest is rich.");
                    harvest * 13 / 10
                }
                _ => harvest,
            };
            self.stock(Item::Grain, harvest);
            if rotated {
                self.stock(Item::Turnip, harvest / 10);
            }
            if potatoes {
                self.stock(Item::Potato, harvest / 8);
            }
            self.note(
                Tone::Good,
                format!("The rye is brought in: {harvest} measures of grain."),
            );
        }

        self.raise_buildings(hands.of(Role::Builder) * WORKDAY * tools / 100);
        self.clothe(members, hands.of(Role::Crafter));
        self.work_out(insight);
        self.keep_accounts(members, &hands);
        self.settle_mood(people, &kinds, hands.of(Role::Idle));
    }

    /// Puts `hours` of builders' work into what is being built.
    fn raise_buildings(&mut self, mut hours: u32) {
        for (id, kind) in self.structures() {
            if hours == 0 {
                break;
            }
            if kind != StructureKind::Site {
                continue;
            }
            let Some(site) = self.state.world.structure_mut(id) else {
                continue;
            };
            let Some(plan) = site.plan().and_then(building) else {
                continue;
            };
            let put_in = hours.min(plan.hours.saturating_sub(site.work()).max(1));
            hours -= put_in;
            if site.build(put_in, plan.hours) {
                self.finished_building(plan.kind);
            }
        }
    }

    /// Has `crafters` sew winter clothes for those who have none, and twist cordage when
    /// there is nobody left to clothe.
    fn clothe(&mut self, members: &[PersonId], crafters: u32) {
        let workshop = self.has_built(StructureKind::Workshop);
        let outfits = crafters * if workshop { 2 } else { 1 };
        let people = members.len() as u32 + 1;

        for _ in 0..outfits {
            let unclothed = members.iter().copied().find(|id| {
                let person = self.state.world.person(*id);
                person.is_some_and(|person| !person.clothed)
            });
            let count = |item| self.store().map_or(0, |store| store.count(item));
            let (pelts, cloth, roots) = (
                count(Item::HarePelt),
                count(Item::Cloth),
                count(Item::SpruceRoot),
            );
            let cordage = count(Item::Cordage);

            if let Some(unclothed) = unclothed.filter(|_| pelts >= PELTS_PER_OUTFIT || cloth >= 2) {
                if let Some(store) = self.store_mut() {
                    if pelts >= PELTS_PER_OUTFIT {
                        store.take(Item::HarePelt, PELTS_PER_OUTFIT);
                    } else {
                        store.take(Item::Cloth, 2);
                    }
                }
                if let Some(person) = self.state.world.person_mut(unclothed) {
                    person.clothed = true;
                }
            } else if roots >= 2
                && cordage < ceiling(Item::Cordage, people)
                && let Some(store) = self.store_mut()
            {
                let cordage = (roots / 2).min(4);
                store.take(Item::SpruceRoot, cordage * 2);
                store.add(Item::Cordage, cordage);
            }
        }
    }

    /// Adds `insight` tenths of a thinker's day to what is being worked out.
    pub(super) fn work_out(&mut self, insight: u32) {
        let learned = self.state.settlement.as_mut().and_then(|settlement| {
            let pursuit = settlement.pursuit?;
            settlement.insight += insight;
            (settlement.insight >= pursuit.cost() * 10).then(|| {
                let age = settlement.age();
                settlement.advances.push(pursuit);
                settlement.pursuit = None;
                settlement.insight = 0;
                (pursuit, age, settlement.age())
            })
        });
        let Some((advance, before, after)) = learned else {
            return;
        };

        self.stale = true;
        self.alert = true;
        self.note(
            Tone::Good,
            format!(
                "Your people have worked out {}. {}.",
                advance.name().to_lowercase(),
                advance.use_of()
            ),
        );
        self.chronicle(format!("{} was worked out.", advance.name()));
        if after != before {
            self.chronicle(format!("The {} began.", after.name()));
            let place = self.state.settlement.as_ref().map(|s| s.name.clone());
            self.note(
                Tone::Good,
                format!(
                    "With that, {} has come into the {}.",
                    place.unwrap_or_default(),
                    after.name()
                ),
            );
        }
    }

    /// Returns the share of what people earn that is raised in taxes, in hundredths, as
    /// the laws let it be.
    pub(super) fn tax_rate(&self) -> u8 {
        let Some(settlement) = &self.state.settlement else {
            return 0;
        };
        let mut tax = settlement.tax.min(HIGHEST_TAX);
        if settlement.lives_by(Law::PublicWorks) {
            tax = tax.max(20);
        }
        if settlement.lives_by(Law::FreeMarket) {
            tax = tax.min(10);
        }
        tax
    }

    /// Raises the taxes, pays those who are paid out of the treasury, and has the
    /// merchants sell what the store can spare and buy what it lacks.
    fn keep_accounts(&mut self, members: &[PersonId], hands: &Hands) {
        let Some(settlement) = &self.state.settlement else {
            return;
        };
        if !settlement.knows(Advance::Coinage) {
            return;
        }
        let people = members.len() as u32 + 1;
        let running = settlement.running;
        let tax = i64::from(self.tax_rate());
        let leads = self.leads();
        let tariffs = settlement.lives_by(Law::Tariffs);
        let banked = self.has_built(StructureKind::Bank);
        let depot = self.has_built(StructureKind::MachineDepot);
        let food_days = self.food_days(people as usize);
        let knows_bronze = settlement.knows(Advance::Bronze);
        let mut orders = settlement.orders.clone();
        let mut treasury = settlement.treasury;
        let mut ledger = Ledger::default();

        // Taxes, of which more is had the more clerks there are to see to it.
        let (mut earned, mut wages) = (0i64, 0i64);
        for role in Role::ALL {
            let pay = i64::from(role.wage()) * i64::from(hands.of(role));
            if role.is_public() {
                wages += pay;
            } else {
                earned += pay;
            }
        }
        wages += i64::from(hands.hired) * i64::from(Role::Thinker.wage());
        if leads {
            wages += STIPEND;
        }
        let reach = (running.clerks * 100 * 100 / people).min(100);
        let collected =
            60 + 40 * i64::from(reach) / 100 + 10 * i64::from(self.office(Office::Treasurer));
        ledger.taxes = earned * tax / 100 * collected / 100;
        treasury += ledger.taxes;
        if banked && treasury > 0 {
            // Three in a hundred a year.
            ledger.interest = treasury / 12_000;
            treasury += ledger.interest;
        }

        // What stands built is kept up first, and those who work for all are paid out
        // of what is left.
        let upkeep: i64 = self
            .built()
            .into_iter()
            .filter_map(building)
            .map(|plan| i64::from(plan.upkeep()))
            .sum();
        let upkeep = upkeep / 100;
        ledger.upkeep = upkeep.min(treasury.max(0));
        treasury -= ledger.upkeep;
        ledger.wages = wages.min(treasury.max(0));
        ledger.unpaid = ledger.wages < wages || ledger.upkeep < upkeep;
        treasury -= ledger.wages;
        if leads && !ledger.unpaid {
            self.state.purse += STIPEND;
        }

        // Trade with whoever comes by, as much of it as there are hands for.
        let mut store = self.store().cloned().unwrap_or_default();
        let mut room = i64::from(running.trade);
        let price = |item: Item, percent: u32| i64::from((item.value() * percent).div_ceil(100));
        if room > 0 {
            for stack in store.stacks().to_vec() {
                let item = stack.item;
                let kept = if item.food().is_some() {
                    if food_days <= FOOD_TO_SPARE || item == Item::Ale {
                        continue;
                    }
                    stack.count - stack.count / 20
                } else {
                    ceiling(item, people) / 3
                };
                let each = price(item, EXPORT_PRICE);
                // Something is always sold of what there is too much of.
                let sold = i64::from(stack.count.saturating_sub(kept)).min((room / each).max(1));
                if sold <= 0 || room <= 0 {
                    continue;
                }
                // What is sold is the gain of those who made it. The treasury has its
                // share of it in taxes, and a tenth more where there are tariffs.
                store.take(item, sold as u32);
                let had = sold * each;
                let share = had * (tax + if tariffs { 10 } else { 0 }) / 100;
                ledger.sales += had;
                ledger.taxes += share;
                treasury += share;
                room -= had;
            }
        }

        // What is always wanted from outside, after what whoever leads has asked for.
        let mut wanted = orders.clone();
        if knows_bronze && !store.has(Item::Tools) {
            wanted.push((Item::Tools, 2));
        }
        if depot && store.count(Item::Oil) < ceiling(Item::Oil, people) {
            wanted.push((Item::Oil, 10));
        }
        if food_days < FOOD_TO_BUY {
            wanted.push((Item::Grain, people * 8));
        }
        let mut room = i64::from(running.trade);
        let percent = if tariffs {
            IMPORT_PRICE * 5 / 4
        } else {
            IMPORT_PRICE
        };
        for (index, (item, count)) in wanted.into_iter().enumerate() {
            let each = price(item, percent);
            // One of anything can be had in a day, however dear it is.
            let bought = i64::from(count)
                .min((room / each).max(1))
                .min(treasury.max(0) / each);
            if bought <= 0 || room <= 0 {
                continue;
            }
            store.add(item, bought as u32);
            ledger.purchases += bought * each;
            treasury -= bought * each;
            room -= bought * each;
            if let Some(order) = orders.get_mut(index) {
                order.1 -= bought as u32;
            }
        }
        orders.retain(|(_, count)| *count > 0);

        if let Some(kept) = self.store_mut() {
            *kept = store;
        }
        if let Some(settlement) = &mut self.state.settlement {
            settlement.treasury = treasury;
            settlement.ledger = ledger;
            settlement.orders = orders;
        }
        self.pay_dividends(|role| hands.of(role));
    }

    /// Lets how content people are drift towards what their lot makes them.
    fn settle_mood(&mut self, people: u32, kinds: &[StructureKind], idle: u32) {
        let housing = self.housing();
        let food_days = self.food_days(people as usize);
        let tax = i32::from(self.tax_rate());
        let Some(settlement) = &self.state.settlement else {
            return;
        };
        let has = |kind| kinds.contains(&kind);
        let running = settlement.running;
        let store = self.store().cloned().unwrap_or_default();
        let fare = store
            .stacks()
            .iter()
            .filter(|stack| stack.item.food().is_some_and(|food| food.energy > 0))
            .filter(|stack| stack.count >= people)
            .count();
        let coined = settlement.knows(Advance::Coinage);

        let mut due: i32 = 50;
        due += match food_days {
            0..5 => -20,
            5..15 => -8,
            15..30 => 0,
            _ => 8,
        };
        due += match fare {
            0..3 => 0,
            3..5 => 4,
            _ => 8,
        };
        due += if people > housing { -12 } else { 4 };
        if running.tavern {
            due += 6;
        }
        if has(StructureKind::Bathhouse) {
            due += 3;
        }
        if has(StructureKind::Well) || has(StructureKind::Waterworks) {
            due += 3;
        }
        if running.healers * PEOPLE_PER_HEALER >= people {
            due += 5;
        }
        if running.teachers > 0 {
            due += 3;
        }
        if running.guards * 80 >= people {
            due += 2;
        }
        due += 2 * self.office(Office::Almoner) as i32;
        // What there is to believe in, to look at, and to do of an evening.
        if has(StructureKind::Sauna) {
            due += 2;
        }
        if running.priests > 0 {
            due += if has(StructureKind::Temple) { 7 } else { 4 };
        }
        if running.performers > 0 {
            let venues = [
                StructureKind::Theatre,
                StructureKind::Cinema,
                StructureKind::Stadium,
            ];
            due += 4 * venues.into_iter().filter(|kind| has(*kind)).count() as i32;
        }
        if has(StructureKind::Park) {
            due += 3;
        }
        if has(StructureKind::Museum) {
            due += 2;
        }
        due -= i32::from(settlement.crime) / 5;
        due -= (running.pollution / 5).min(12) as i32;
        if running.power {
            due += 5;
        }
        if people >= 60 && !has(StructureKind::Waterworks) {
            due -= 5;
        }
        // Someone has to keep house, but where more than a third of those who could
        // work have nothing to do, it tells on everyone.
        if people >= 10 {
            let spare = (idle * 3).saturating_sub(people);
            due -= (spare * 30 / people).min(15) as i32;
        }
        if coined {
            due -= (tax - 10) * 4 / 5;
            if settlement.ledger.unpaid {
                due -= 8;
            }
        }

        let mood = i32::from(settlement.contentment);
        let mood = (mood + (due.clamp(0, 100) - mood).signum()) as u8;

        // Crime grows where people are idle, crowded, and discontented, and where a
        // place is too big for everyone to know everyone. Guards and a court keep it
        // down.
        let mut wrong: i32 = 5 + (people / 60).min(20) as i32;
        if people >= 10 {
            wrong += (idle * 60 / people).min(30) as i32;
        }
        if people > housing {
            wrong += 10;
        }
        if mood < 40 {
            wrong += 15;
        }
        if running.guards * 80 >= people {
            wrong -= 15;
        }
        if has(StructureKind::Courthouse) && running.clerks > 0 {
            wrong -= 10;
        }
        if running.priests > 0 {
            wrong -= 3;
        }
        let crime = i32::from(settlement.crime);
        let crime = (crime + (wrong.clamp(0, 100) - crime).signum()) as u8;
        if let Some(settlement) = &mut self.state.settlement {
            settlement.contentment = mood;
            settlement.crime = crime;
        }
        self.thieve(people, crime);
    }

    /// Has thieves make off with some of what is in the store and the treasury, the more
    /// often the more crime there is.
    fn thieve(&mut self, people: u32, crime: u8) {
        if people < 20 || !self.state.rng.one_in(u64::from(crime), 3_000) {
            return;
        }
        let stolen = self.store_mut().and_then(|store| {
            let richest = store
                .stacks()
                .iter()
                .filter(|stack| stack.count >= 20)
                .max_by_key(|stack| u64::from(stack.count) * u64::from(stack.item.value()))
                .copied()?;
            let taken = store.take(richest.item, richest.count / 20);
            Some((taken.item, taken.count))
        });
        if let Some(settlement) = &mut self.state.settlement {
            settlement.treasury -= settlement.treasury.max(0) / 50;
        }
        if let Some((item, count)) = stolen {
            self.note(
                Tone::Bad,
                format!(
                    "Thieves have been at the store: {count} × {} are gone.",
                    item.name().to_lowercase()
                ),
            );
        }
    }

    /// Returns what `count` of `item` cost the player character at the market.
    pub(super) fn buying_price(item: Item, count: u32) -> i64 {
        (i64::from(item.value()) * i64::from(count) * BUYING_PRICE + 99) / 100
    }

    /// Returns what the player character is paid for `stack` at the market.
    pub(super) fn selling_price(stack: &Stack) -> i64 {
        i64::from(stack.item.value()) * i64::from(stack.count) * SELLING_PRICE / 100
    }

    /// Takes `coins` out of the purse of the player character and puts them in the
    /// treasury, for something had from the store. The purse may go into debt, which
    /// people take note of.
    pub(super) fn charge(&mut self, coins: i64) {
        let before = self.state.purse;
        self.state.purse -= coins;
        if let Some(settlement) = &mut self.state.settlement {
            settlement.treasury += coins;
        }
        self.society_stale = true;

        let owed = |purse: i64| (-purse).max(0) / 50;
        if owed(self.state.purse) > owed(before) {
            for id in self.members() {
                if let Some(person) = self.state.world.person_mut(id) {
                    person.cool_to(1);
                }
            }
            self.note(
                Tone::Bad,
                format!(
                    "You owe the market {} coins, and people take note.",
                    -self.state.purse
                ),
            );
        }
    }

    /// Pays the player character `coins` out of the treasury, as far as it goes. Returns
    /// how many coins were paid.
    pub(super) fn pay(&mut self, coins: i64) -> i64 {
        let Some(settlement) = &mut self.state.settlement else {
            return 0;
        };
        let paid = coins.min(settlement.treasury.max(0));
        settlement.treasury -= paid;
        self.state.purse += paid;
        self.society_stale = true;
        paid
    }

    pub(super) fn set_tax(&mut self, tax: u8) {
        if let Err(reason) = self.authority() {
            self.note(Tone::Plain, reason);
            return;
        }
        if let Some(settlement) = &mut self.state.settlement {
            settlement.tax = tax.min(HIGHEST_TAX);
            self.society_stale = true;
        }
    }

    /// Asks the merchants to buy `count` of `item` from outside in all, or no more of it
    /// if that is none.
    pub(super) fn order(&mut self, item: Item, count: u32) {
        if let Err(reason) = self.authority() {
            self.note(Tone::Plain, reason);
            return;
        }
        self.place_order(item, count);
    }

    pub(super) fn place_order(&mut self, item: Item, count: u32) {
        let Some(settlement) = &mut self.state.settlement else {
            return;
        };
        settlement.orders.retain(|(ordered, _)| *ordered != item);
        if count > 0 {
            settlement.orders.push((item, count.min(10_000)));
        }
        self.society_stale = true;
    }

    /// Returns why the player cannot work a shift at the building, if they cannot.
    pub(super) fn check_labor(&self, id: ThingId) -> Result<(), String> {
        let kind = self
            .state
            .world
            .structure(id)
            .map(|structure| structure.kind());
        let jobs = kind.and_then(building).and_then(|plan| plan.jobs);
        let Some((role, _)) = jobs else {
            return Err("There is no work to be had there.".to_owned());
        };
        if self.state.settlement.is_none() {
            return Err("There is no settlement to work for.".to_owned());
        }
        let date = Date::from_timestamp(self.state.time);
        let growing = (GROWING_SEASON.0..=GROWING_SEASON.1).contains(&date.ordinal());
        if role == Role::Farmer && !growing {
            return Err("Nothing grows at this time of the year.".to_owned());
        }
        Ok(())
    }

    /// Brings about what half a day of the player character's work at the building comes
    /// to: half of what one of its trade does in a day, and half a day's pay.
    pub(super) fn finish_labor(&mut self, id: ThingId) -> bool {
        let kind = self
            .state
            .world
            .structure(id)
            .map(|structure| structure.kind());
        let Some((role, _)) = kind.and_then(building).and_then(|plan| plan.jobs) else {
            return false;
        };
        let kinds = self.built();
        let people = self.members().len() as u32 + 1;
        let tools = self.tools_percent();
        let percent = self.yield_percent(role, &kinds, tools) / 2;

        let mut store = self.store().cloned().unwrap_or_default();
        let supplied = self.ply(role, 1, percent, people, &mut store) > 0;
        if let Some(kept) = self.store_mut() {
            *kept = store;
        }
        match role {
            Role::Farmer => {
                let grown = GRAIN_PER_FARMER_DAY * self.farm_percent(tools) / 200;
                if let Some(settlement) = &mut self.state.settlement {
                    settlement.growing += grown;
                }
            }
            Role::Scholar => self.work_out(15),
            _ => {}
        }

        let wage = i64::from(role.wage().div_ceil(2));
        let paid = if self.monetized() { self.pay(wage) } else { 0 };
        if paid < wage {
            // Work that is not paid for in coin is remembered.
            self.credit(4 * super::politics::HOUR_OF_WORK);
        }
        self.stale = true;
        self.society_stale = true;

        let name = role.name().to_lowercase();
        let words = match (supplied, paid) {
            (false, _) => {
                format!("You turn up for a shift as a {name}, but there is nothing to work with.")
            }
            (true, 0) => format!("You work a shift as a {name}."),
            (true, paid) => format!("You work a shift as a {name}, and are paid {paid} coins."),
        };
        self.note(Tone::Plain, words);
        supplied
    }
}

#[cfg(test)]
mod tests;
