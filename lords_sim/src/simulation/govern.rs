//! How a settlement runs itself when the player does not run it: who does what, what is
//! built, what is worked out, what is bought, and what is taxed.

use crate::item::{Inventory, Item};
use crate::people::{PersonId, Role};
use crate::politics::{Era, Law};
use crate::settlement::{Advance, Building, FARMERS_PER_FIELD, building, places};
use crate::time::gregorian::Date;
use crate::work::ceiling;
use crate::world::{PlantSpecies, StructureKind};

use super::Simulation;
use super::economy::{
    FISHING_GROUNDS, FORAGING_GROUNDS, GROWING_SEASON, HUNTING_GROUNDS, PEOPLE_PER_HEALER,
    TRAPPING_GROUNDS,
};

/// How many days of food there must be in the store before anyone is spared from finding
/// more of it, in the half of the year when food is easily found and in the half when
/// the winter has to be laid in for, and how many more before anyone is set to thinking.
const SPARE_DAYS: (u32, u32) = (15, 45);
const THINKING_DAYS: u32 = 30;

/// The kilocalories a day someone needs, with something over for the young and for what
/// goes off, and what one who fishes or traps brings in.
const NEED: u32 = 2_700;
const FINDINGS: u32 = 4_500;
const HERDING: u32 = 4_600;

/// What one farmer grows in a year with nothing to help them, in kilocalories a day.
const FARMING: u32 = 138 * 30 * 340 / 365;

/// The most building sites there are at once.
const MOST_SITES: u32 = 4;

/// The houses there are, the best first.
const HOUSES: [StructureKind; 6] = [
    StructureKind::ApartmentBlock,
    StructureKind::Tenement,
    StructureKind::Townhouse,
    StructureKind::Cottage,
    StructureKind::Longhouse,
    StructureKind::Hut,
];

/// The trades that take a workplace, in the order they are manned when there are not
/// hands enough for all of them.
const TRADES: [Role; 29] = [
    Role::Smith,
    Role::Burner,
    Role::Miner,
    Role::Quarrier,
    Role::Sawyer,
    Role::Brickmaker,
    Role::Healer,
    Role::Guard,
    Role::Merchant,
    Role::Miller,
    Role::Glassmaker,
    Role::Weaver,
    Role::Brewer,
    Role::Innkeeper,
    Role::Teacher,
    Role::Clerk,
    Role::Sailor,
    Role::Steelworker,
    Role::FactoryHand,
    Role::Engineer,
    Role::MillHand,
    Role::Printer,
    Role::Canner,
    Role::Mechanic,
    Role::Operator,
    Role::Carrier,
    Role::Priest,
    Role::Performer,
    Role::Scholar,
];

/// What is built apart from houses, the most pressing first, with the fewest people a
/// settlement has before it builds one, and how many people there are to each one more
/// of it, if more than one is ever built.
const WORKS: [(StructureKind, u32, Option<u32>); 57] = [
    (StructureKind::Smokehouse, 0, None),
    (StructureKind::Storehouse, 0, None),
    (StructureKind::Pasture, 6, Some(40)),
    (StructureKind::Workshop, 8, None),
    (StructureKind::Dock, 6, None),
    (StructureKind::Kiln, 8, None),
    (StructureKind::Hall, Era::VILLAGE, None),
    (StructureKind::Well, 8, None),
    (StructureKind::Palisade, 12, None),
    (StructureKind::Quarry, 12, Some(200)),
    (StructureKind::CharcoalKiln, 12, Some(100)),
    (StructureKind::Mine, 14, Some(150)),
    (StructureKind::Smithy, 14, Some(120)),
    (StructureKind::Barracks, 18, Some(400)),
    (StructureKind::Brewery, 16, Some(120)),
    (StructureKind::Weavery, 16, Some(150)),
    (StructureKind::Market, 16, Some(250)),
    (StructureKind::Mill, 16, None),
    (StructureKind::Sawmill, 16, Some(150)),
    (StructureKind::Infirmary, 20, Some(180)),
    (StructureKind::School, 20, Some(200)),
    (StructureKind::Tavern, 20, None),
    (StructureKind::Bathhouse, 30, None),
    (StructureKind::Brickworks, 20, Some(150)),
    (StructureKind::Glassworks, 25, None),
    (StructureKind::Courthouse, 25, None),
    (StructureKind::Wall, 150, None),
    (StructureKind::Harbour, 30, Some(800)),
    (StructureKind::Library, 25, None),
    (StructureKind::Guildhall, 30, None),
    (StructureKind::TownHall, 40, None),
    (StructureKind::Bank, 40, None),
    (StructureKind::PrintingHouse, 30, None),
    (StructureKind::Waterworks, 50, None),
    (StructureKind::University, 40, None),
    (StructureKind::Steelworks, 40, Some(300)),
    (StructureKind::Factory, 50, Some(300)),
    (StructureKind::TextileMill, 60, None),
    (StructureKind::RailwayStation, 60, None),
    (StructureKind::Cannery, 60, None),
    (StructureKind::PowerStation, 60, None),
    (StructureKind::TelegraphOffice, 60, None),
    (StructureKind::MachineDepot, 60, None),
    (StructureKind::Hospital, 80, None),
    (StructureKind::ColdStore, 60, None),
    (StructureKind::BroadcastingHouse, 80, None),
    (StructureKind::Institute, 80, None),
    (StructureKind::ComputingCentre, 80, None),
    (StructureKind::Airport, 150, None),
    (StructureKind::Sauna, 6, None),
    (StructureKind::Shrine, 10, None),
    (StructureKind::Temple, 60, None),
    (StructureKind::Theatre, 80, None),
    (StructureKind::Park, 100, None),
    (StructureKind::Museum, 120, None),
    (StructureKind::Cinema, 150, None),
    (StructureKind::Stadium, 300, None),
];

impl Simulation {
    /// Runs the settlement the way it runs itself: sees to it that enough people find
    /// food, that what is needed gets built, and that something is being worked out.
    pub(super) fn govern(&mut self) {
        let now = self.state.time;
        let date = Date::from_timestamp(now);
        let working_age = self.working_age();
        let members = self.members();
        let workers: Vec<(PersonId, Role)> = members
            .iter()
            .filter_map(|id| {
                let person = self.state.world.person(*id)?;
                let hired = person.hired_until.is_some_and(|until| until > now);
                (person.age(now) >= working_age && !hired).then_some((*id, person.role))
            })
            .collect();
        let hands = workers.len() as u32;
        let people = members.len() as u32 + 1;
        let food_days = self.food_days(people as usize);
        let kinds = self.built();
        // What is being built counts as built when it comes to what else is wanted.
        let planned: Vec<StructureKind> = self
            .structures()
            .iter()
            .filter(|(_, kind)| *kind == StructureKind::Site)
            .filter_map(|(id, _)| self.state.world.structure(*id)?.plan())
            .collect();
        let sites = planned.len() as u32;
        let count = |kind| {
            let all = kinds.iter().chain(&planned);
            all.filter(|built| **built == kind).count() as u32
        };
        let store = self.store().cloned().unwrap_or_default();
        let Some(settlement) = &self.state.settlement else {
            return;
        };
        let knows = |advance| settlement.knows(advance);
        // From August to February the winter is laid in for.
        let spare = if (3..8).contains(&date.month()) {
            SPARE_DAYS.0
        } else {
            SPARE_DAYS.1
        };
        let lean = food_days < spare;
        let growing = (GROWING_SEASON.0..=GROWING_SEASON.1).contains(&date.ordinal());
        let berries =
            PlantSpecies::Bilberry.is_ripe(date) || PlantSpecies::Lingonberry.is_ripe(date);
        let unclothed = members.iter().any(|id| {
            let person = self.state.world.person(*id);
            person.is_some_and(|person| !person.clothed)
        });
        let short = |item: Item| store.count(item) < ceiling(item, people) / 2;

        // How much of what is eaten the fields and the pastures bring in, and how many
        // have to fish, trap, and forage for the rest.
        let farm = u64::from(self.farm_percent(settlement.tool_percent()));
        let fields = places(Role::Farmer, &kinds).unwrap_or(0);
        let pastures = places(Role::Herder, &kinds).unwrap_or(0);
        let herders = pastures.min(hands / 4);
        // No more is brought in than is eaten, and far less while the barns are full.
        let needed = u64::from(people) * u64::from(NEED);
        let needed = match food_days {
            0..120 => needed * 13 / 10,
            120..365 => needed * 11 / 10,
            365..730 => needed * 6 / 10,
            _ => needed / 5,
        };
        let by_a_farmer = (u64::from(FARMING) * farm / 100).max(1);
        let to_grow = needed.saturating_sub(u64::from(herders * HERDING));
        let farmers_needed = to_grow.div_ceil(by_a_farmer) as u32;
        let farmers = fields.min(hands * 7 / 10).min(farmers_needed);
        let farmed = u64::from(farmers) * by_a_farmer + u64::from(herders * HERDING);
        let wild = needed.saturating_sub(farmed) as u32;
        let ample = food_days >= spare + THINKING_DAYS;
        let finders = if lean {
            hands
        } else if ample {
            wild / FINDINGS
        } else {
            wild.div_ceil(FINDINGS)
        };
        // Nobody is kept to think while those the treasury pays already go unpaid.
        let thinking = ample && settlement.pursuit.is_some() && !settlement.ledger.unpaid;
        let waters = FISHING_GROUNDS * (1 + count(StructureKind::Dock).min(1))
            + 2 * FISHING_GROUNDS * count(StructureKind::Harbour).min(1);

        let threat = self
            .state
            .neighbours
            .iter()
            .filter(|neighbour| neighbour.is_hostile())
            .map(|neighbour| neighbour.raiders())
            .max()
            .unwrap_or(0);
        let walls =
            6 * count(StructureKind::Palisade).min(1) + 20 * count(StructureKind::Wall).min(1);

        // Who does what, the most pressing first.
        let mut wanted: Vec<(Role, u32)> = Vec::new();
        let mut want = |role: Role, many: u32| {
            if many > 0 {
                wanted.push((role, many));
            }
        };
        // Logs heat the houses through the winter, and feed the kilns and the saws.
        let burning = places(Role::Burner, &kinds).unwrap_or(0) * 2
            + places(Role::Sawyer, &kinds).unwrap_or(0) * 3
            + places(Role::Brickmaker, &kinds).unwrap_or(0);
        let felled = (4 * settlement.tool_percent() / 100).max(1);
        if hands >= 3 && (store.count(Item::Log) < ceiling(Item::Log, people) || sites > 0) {
            want(Role::Woodcutter, 1 + (people / 8 + burning) / felled);
        }
        if growing {
            want(Role::Farmer, farmers);
        }
        want(Role::Herder, herders);
        let hunters = (finders / 6).min(HUNTING_GROUNDS);
        let finders = finders - hunters;
        let trappers = (finders / 3).min(TRAPPING_GROUNDS);
        let foragers = if berries {
            (finders / 4).min(FORAGING_GROUNDS)
        } else {
            0
        };
        let fishers = (finders - trappers - foragers).min(waters);
        want(Role::Fisher, fishers);
        want(Role::Trapper, trappers);
        want(Role::Hunter, hunters);
        want(Role::Forager, foragers);
        if !lean {
            if sites > 0 {
                want(Role::Builder, (hands / 5).max(1).min(sites * 6));
            }
            // There is always someone who thinks, where there is anyone to spare.
            if thinking && hands >= 5 {
                want(Role::Thinker, 1);
            }
            if unclothed && (store.count(Item::HarePelt) >= 4 || store.count(Item::Cloth) >= 2) {
                want(Role::Crafter, 1);
            }
            let materials = [Item::Stick, Item::SpruceBough, Item::Stone, Item::Clay];
            if hands >= 5 && materials.into_iter().any(short) {
                want(Role::Gatherer, hands / 12 + 1);
            }
            for role in TRADES {
                let places = places(role, &kinds).unwrap_or(0);
                let places = match role {
                    // No more healers and teachers than there are people to see to.
                    Role::Healer => places.min(people.div_ceil(PEOPLE_PER_HEALER)),
                    Role::Teacher => places.min(people.div_ceil(100)),
                    // And no more guards than there are raiders to fear.
                    // nor more than one in twenty-five can be spared for.
                    Role::Guard => places
                        .min(threat.saturating_sub(walls).max(2))
                        .min(people / 25 + 1),
                    Role::Scholar if settlement.pursuit.is_none() => 0,
                    _ => places,
                };
                want(role, places);
            }
            if thinking {
                want(Role::Thinker, (hands / 15).max(1) - u32::from(hands >= 5));
            }
        }

        // Those who do what is wanted already keep at it, and the rest fill in.
        let mut assigned: Vec<(PersonId, Role)> = Vec::new();
        let mut free: Vec<PersonId> = Vec::new();
        let mut open = wanted.clone();
        let mut left = hands;
        // What is asked for is cut to the hands there are, the most pressing first.
        for (_, many) in &mut open {
            *many = (*many).min(left);
            left -= *many;
        }
        for (id, role) in &workers {
            match open
                .iter_mut()
                .find(|(wanted, many)| wanted == role && *many > 0)
            {
                Some((_, many)) => *many -= 1,
                None => free.push(*id),
            }
        }
        // Whoever is left over finds food while it is short. Otherwise some of them
        // think, and the rest have nothing to do.
        let finding = [Role::Fisher, Role::Trapper, Role::Fisher];
        let mut spare_minds = if thinking { hands / 40 } else { 0 };
        for (index, id) in free.into_iter().enumerate() {
            let role = match open.iter_mut().find(|(_, many)| *many > 0) {
                Some((role, many)) => {
                    *many -= 1;
                    *role
                }
                None if !ample => finding[index % finding.len()],
                None if spare_minds > 0 => {
                    spare_minds -= 1;
                    Role::Thinker
                }
                None => Role::Idle,
            };
            assigned.push((id, role));
        }

        // What to work out, if nothing is: whatever comes the easiest.
        let pursuit = Advance::ALL
            .into_iter()
            .filter(|advance| settlement.can_pursue(*advance))
            .min_by_key(|advance| advance.cost());
        let idle = settlement.pursuit.is_none();

        // What to build.
        let coined = knows(Advance::Coinage);
        let free_works = settlement.lives_by(Law::PublicWorks);
        let treasury = settlement.treasury;
        let trading = settlement.running.trade > 0;
        let unpaid = settlement.ledger.unpaid;
        let tax = settlement.tax;
        let housing = self.housing();
        let known = |plan: &Building| plan.needs.is_none_or(knows);
        let stocked = |plan: &Building| {
            let materials = plan.materials.iter();
            materials
                .clone()
                .all(|(item, count)| store.count(*item) >= *count)
        };
        let afforded = |plan: &Building| !coined || free_works || i64::from(plan.coins) <= treasury;
        // Fields are cleared for what a year takes, whatever is in the barns today.
        let yearly = (u64::from(people) * u64::from(NEED) * 11 / 10)
            .saturating_sub(u64::from(herders * HERDING));
        let fields_wanted = (yearly.div_ceil(by_a_farmer) as u32)
            .div_ceil(FARMERS_PER_FIELD)
            .min(hands * 7 / 30 + 1);
        // Another workplace of a trade is wanted only while what the trade makes is short.
        let short_of = |kind: StructureKind| {
            let Some((role, _)) = building(kind).and_then(|plan| plan.jobs) else {
                return true;
            };
            // Trades that make nothing to put in the store are wanted by the head.
            let mut main = crate::work::PROCESSES
                .iter()
                .filter(|process| process.role == role && !process.aside)
                .filter_map(|process| process.makes.first())
                .peekable();
            main.peek().is_none() || main.any(|(item, _)| short(*item))
        };
        let wants = |kind: StructureKind, least: u32, each: Option<u32>| {
            if people < least {
                return false;
            }
            let most = match kind {
                StructureKind::Field => fields_wanted,
                StructureKind::Pasture => 1 + (people / 40).min(5),
                _ => each.map_or(1, |each| 1 + people / each),
            };
            let built = count(kind);
            built < most && (built == 0 || short_of(kind))
        };

        let mut plans: Vec<StructureKind> = Vec::new();
        let rising: u32 = planned
            .iter()
            .map(|kind| crate::settlement::houses(*kind))
            .sum();
        if housing + rising < people + 2 + people / 10 {
            // The best house known is built, or the next best while that cannot be. Only
            // when people are left without a roof does anything do.
            let buildable = |kind: &StructureKind| {
                building(*kind).is_some_and(|plan| known(plan) && stocked(plan) && afforded(plan))
            };
            let within_reach: Vec<StructureKind> = HOUSES
                .into_iter()
                .filter(|kind| building(*kind).is_some_and(known))
                .collect();
            let house = if housing < people {
                within_reach.iter().copied().find(buildable)
            } else {
                within_reach.iter().copied().take(2).find(buildable)
            };
            plans.extend(house.or(within_reach.first().copied()));
        }
        if knows(Advance::Farming) && wants(StructureKind::Field, 0, None) {
            plans.push(StructureKind::Field);
        }
        // What there is none of yet comes before more of what there is.
        let mut more: Vec<StructureKind> = Vec::new();
        for (kind, least, each) in WORKS {
            if building(kind).is_some_and(known) && wants(kind, least, each) {
                if count(kind) == 0 {
                    plans.push(kind);
                } else {
                    more.push(kind);
                }
            }
        }
        plans.extend(more);

        let room = MOST_SITES.min(1 + hands / 12).saturating_sub(sites);
        let mut to_build: Vec<StructureKind> = Vec::new();
        let mut spent = Inventory::default();
        let mut coins = 0;
        let mut orders: Vec<(Item, u32)> = Vec::new();
        let mut saving = false;
        for kind in plans {
            let Some(plan) = building(kind) else {
                continue;
            };
            let have = |item: Item| store.count(item).saturating_sub(spent.count(item));
            let materials = plan.materials.iter();
            let supplied = materials.clone().all(|(item, count)| have(*item) >= *count);
            // Nothing that costs coin is built while those the treasury pays go unpaid.
            let paid = !coined
                || plan.coins == 0
                || (!unpaid && (free_works || i64::from(plan.coins) + coins <= treasury));

            if supplied && paid && (to_build.len() as u32) < room && !(lean && people >= 4) {
                for (item, count) in plan.materials {
                    spent.add(*item, *count);
                }
                coins += i64::from(plan.coins);
                to_build.push(kind);
            } else if supplied && !paid {
                // Something stands ready to be built but for the coin.
                saving = true;
            } else if !supplied && orders.is_empty() && trading {
                // What nobody here makes yet is sent for.
                for (item, count) in plan.materials {
                    let made_here = Role::ALL.into_iter().any(|role| {
                        let makes = role.makes().iter().any(|(made, _)| made == item);
                        makes && places(role, &kinds).is_none_or(|places| places > 0)
                    });
                    if !made_here && have(*item) < *count {
                        orders.push((*item, count - have(*item)));
                    }
                }
            }
        }

        // What to tax: more when those the treasury pays go unpaid, and less when it
        // has more than it knows what to do with.
        let tax = if !coined || date.day() != 1 {
            tax
        } else if unpaid && tax < 30 {
            tax + 2
        } else if saving && tax < 25 {
            // What is wanted and cannot be paid for is saved up for.
            tax + 1
        } else if !unpaid && !saving && tax > 2 && treasury > 200 * i64::from(people) {
            tax - 1
        } else {
            tax
        };

        for (id, role) in assigned {
            if let Some(person) = self.state.world.person_mut(id)
                && person.role != role
            {
                person.take_up(role);
                self.society_stale = true;
            }
        }
        if let Some(settlement) = &mut self.state.settlement {
            if idle && let Some(pursuit) = pursuit {
                settlement.pursuit = Some(pursuit);
                settlement.insight = 0;
            }
            if settlement.tax != tax {
                settlement.tax = tax;
            }
            if trading {
                settlement.orders = orders;
            }
            self.society_stale = true;
        }
        for kind in to_build {
            let _ = self.mark_out(kind);
        }
        // A treasury with far more in it than there is any use for is spent on those who
        // think, a hundredth of what is over a day.
        let over = treasury - 500 * i64::from(people);
        if coined && over > 0 && !idle {
            let spent = (over / 100).min(50 * i64::from(people));
            if let Some(settlement) = &mut self.state.settlement {
                settlement.treasury -= spent;
            }
            self.work_out(spent as u32 * 2);
        }

        // A treasury that stays empty under a law that keeps taxes low gives the law up.
        if unpaid && date.day() == 1 && self.lives_by(Law::FreeMarket) && !self.leads() {
            if let Some(settlement) = &mut self.state.settlement {
                settlement.laws.retain(|law| *law != Law::FreeMarket);
            }
            self.note(
                crate::journal::Tone::Plain,
                "With the treasury empty, the free market is the law no longer.",
            );
        }
    }
}
