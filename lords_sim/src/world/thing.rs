use serde::{Deserialize, Serialize};

use crate::item::{Inventory, Item, Stack};
use crate::object::Object;
use crate::space::{Length, Location};
use crate::time::Timestamp;
use crate::time::gregorian::{Date, SECONDS_PER_DAY};
use crate::world::Kind;

/// How many days it takes a tree to drop another stick's worth of dead wood.
const DAYS_PER_STICK: u64 = 5;

/// The trunk diameter from which a tree has bark and wood worth taking.
const GROWN_TRUNK: Length = Length::from_millimeters(120);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Thing {
    Plant(Plant),
    Rock(Rock),
    Pile(Pile),
    Structure(Structure),
}

impl Thing {
    pub fn object(&self) -> &Object {
        match self {
            Self::Plant(plant) => &plant.object,
            Self::Rock(rock) => &rock.object,
            Self::Pile(pile) => &pile.object,
            Self::Structure(structure) => &structure.object,
        }
    }

    pub fn location(&self) -> Location {
        self.object().location()
    }

    pub fn kind(&self) -> Kind {
        match self {
            Self::Plant(plant) => Kind::Plant(plant.species),
            Self::Rock(_) => Kind::Rock,
            Self::Pile(pile) => Kind::Pile(pile.main_item()),
            Self::Structure(structure) => Kind::Structure(structure.kind),
        }
    }

    /// Returns the radius of the ground this thing stands on.
    pub fn footprint(&self) -> Length {
        let diameter = match self {
            Self::Plant(plant) => plant.trunk_diameter,
            Self::Rock(rock) => rock.diameter,
            Self::Pile(_) => Pile::DIAMETER,
            Self::Structure(structure) => structure.kind.diameter(),
        };
        diameter / 2
    }

    pub fn height(&self) -> Length {
        match self {
            Self::Plant(plant) => plant.height,
            Self::Rock(rock) => rock.height,
            Self::Pile(_) => Pile::HEIGHT,
            Self::Structure(structure) => structure.kind.height(),
        }
    }
}

/// A tree, or a patch of shrubs, for which `trunk_diameter` is the width of the patch.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Plant {
    object: Object,
    species: PlantSpecies,
    height: Length,
    trunk_diameter: Length,
    /// How much of what the plant yields had been taken at `taken_at`.
    #[serde(default)]
    taken: u32,
    #[serde(default)]
    taken_at: Timestamp,
    /// Whether the bark has been taken.
    #[serde(default)]
    stripped: bool,
}

impl Plant {
    pub fn new(
        object: Object,
        species: PlantSpecies,
        height: Length,
        trunk_diameter: Length,
    ) -> Self {
        Self {
            object,
            species,
            height,
            trunk_diameter,
            taken: 0,
            taken_at: Timestamp::EPOCH,
            stripped: false,
        }
    }

    pub fn location(&self) -> Location {
        self.object.location()
    }

    pub fn species(&self) -> PlantSpecies {
        self.species
    }

    pub fn height(&self) -> Length {
        self.height
    }

    pub fn trunk_diameter(&self) -> Length {
        self.trunk_diameter
    }

    pub fn is_stripped(&self) -> bool {
        self.stripped
    }

    /// Returns whether the plant is a tree big enough to have bark and logs worth taking.
    pub fn is_grown(&self) -> bool {
        self.species.is_tree() && self.trunk_diameter >= GROWN_TRUNK
    }

    /// Returns how many sticks of dead wood, for a tree, or helpings of berries when they
    /// are ripe, for a patch of shrubs, the plant has to give at `now`.
    pub fn stock(&self, now: Timestamp) -> u32 {
        let most = self.most();

        if self.species.is_tree() {
            let days = now.seconds_since(self.taken_at) / SECONDS_PER_DAY;
            let regrown = u32::try_from(days / DAYS_PER_STICK).unwrap_or(u32::MAX);
            most.saturating_sub(self.taken.saturating_sub(regrown))
        } else if Date::from_timestamp(now).year() == Date::from_timestamp(self.taken_at).year() {
            most.saturating_sub(self.taken)
        } else {
            most
        }
    }

    /// Returns how many logs and sticks felling the plant gives.
    pub fn timber(&self) -> (u32, u32) {
        let diameter = self.trunk_diameter.as_millimeters() as u32;
        let logs = if self.is_grown() {
            (self.height.as_millimeters() as u32 / 4_000).max(1)
        } else {
            0
        };

        (logs, 4 + diameter / 40)
    }

    fn most(&self) -> u32 {
        let diameter = self.trunk_diameter.as_millimeters() as u32;
        if self.species.is_tree() {
            6 + diameter / 50
        } else {
            diameter / 100
        }
    }

    /// Takes `count` of what [`Self::stock`] counts.
    pub(crate) fn take(&mut self, count: u32, now: Timestamp) {
        let left = self.stock(now).saturating_sub(count);
        self.taken = self.most() - left;
        self.taken_at = now;
    }

    pub(crate) fn strip(&mut self) {
        self.stripped = true;
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum PlantSpecies {
    Spruce,
    Pine,
    Birch,
    Bilberry,
    Lingonberry,
}

impl PlantSpecies {
    pub const fn is_tree(self) -> bool {
        matches!(self, Self::Spruce | Self::Pine | Self::Birch)
    }

    /// Returns the berries the species bears, if it bears any.
    pub const fn berries(self) -> Option<Item> {
        match self {
            Self::Bilberry => Some(Item::Bilberries),
            Self::Lingonberry => Some(Item::Lingonberries),
            Self::Spruce | Self::Pine | Self::Birch => None,
        }
    }

    /// Returns whether the berries of the species are ripe on `date`.
    pub const fn is_ripe(self, date: Date) -> bool {
        let day = date.ordinal();
        match self {
            // From the middle of July to the middle of September.
            Self::Bilberry => matches!(day, 196..=258),
            // From late August to the end of October.
            Self::Lingonberry => matches!(day, 232..=304),
            Self::Spruce | Self::Pine | Self::Birch => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rock {
    object: Object,
    diameter: Length,
    height: Length,
}

impl Rock {
    pub fn new(object: Object, diameter: Length, height: Length) -> Self {
        Self {
            object,
            diameter,
            height,
        }
    }

    pub fn diameter(&self) -> Length {
        self.diameter
    }

    pub fn height(&self) -> Length {
        self.height
    }
}

/// Items lying on the ground in one place.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pile {
    object: Object,
    items: Inventory,
}

impl Pile {
    const DIAMETER: Length = Length::from_millimeters(400);
    const HEIGHT: Length = Length::from_millimeters(200);

    pub fn new(object: Object, stacks: impl IntoIterator<Item = Stack>) -> Self {
        Self {
            object,
            items: stacks.into_iter().collect(),
        }
    }

    pub fn items(&self) -> &Inventory {
        &self.items
    }

    pub(crate) fn items_mut(&mut self) -> &mut Inventory {
        &mut self.items
    }

    /// Returns the item the pile has the most of by weight, which is what it looks to be
    /// a pile of.
    pub fn main_item(&self) -> Item {
        self.items
            .stacks()
            .iter()
            .max_by_key(|stack| stack.weight())
            .map_or(Item::Stick, |stack| stack.item)
    }
}

/// Something built or set up by a person.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Structure {
    object: Object,
    kind: StructureKind,
    /// When a fire burns out.
    #[serde(default)]
    until: Timestamp,
    /// When a trap was set, or food hung up to dry.
    #[serde(default)]
    since: Timestamp,
    /// What a trap has caught, or what hangs on a rack.
    #[serde(default)]
    hold: Inventory,
}

impl Structure {
    /// The longest a fire can be stoked to burn for, in in-game seconds.
    pub const LONGEST_BURN: u64 = 12 * 3_600;

    pub fn new(object: Object, kind: StructureKind) -> Self {
        Self {
            object,
            kind,
            until: Timestamp::EPOCH,
            since: object.creation(),
            hold: Inventory::default(),
        }
    }

    pub fn location(&self) -> Location {
        self.object.location()
    }

    pub fn kind(&self) -> StructureKind {
        self.kind
    }

    pub fn until(&self) -> Timestamp {
        self.until
    }

    pub fn since(&self) -> Timestamp {
        self.since
    }

    pub fn hold(&self) -> &Inventory {
        &self.hold
    }

    /// Returns how many more in-game seconds' worth of wood a fire takes at `now`.
    pub fn room(&self, now: Timestamp) -> u64 {
        Self::LONGEST_BURN.saturating_sub(self.until.seconds_since(now))
    }

    pub(crate) fn hold_mut(&mut self) -> &mut Inventory {
        &mut self.hold
    }

    pub(crate) fn set_kind(&mut self, kind: StructureKind) {
        self.kind = kind;
    }

    pub(crate) fn set_since(&mut self, since: Timestamp) {
        self.since = since;
    }

    /// Makes a fire burn `seconds` longer than it would, counted from `now` if it is out,
    /// and no longer than [`Self::LONGEST_BURN`] from `now`.
    pub(crate) fn stoke(&mut self, seconds: u64, now: Timestamp) {
        let from = self.until.max(now).as_seconds();
        let limit = now.as_seconds() + Self::LONGEST_BURN;
        self.until = Timestamp::from_seconds((from + seconds).min(limit));
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum StructureKind {
    /// A burning fire.
    Campfire,
    /// A fire that has gone out.
    Fireplace,
    LeanTo,
    Bed,
    Cabin,
    Snare,
    /// A snare with a catch in it.
    SprungSnare,
    FishTrap,
    DryingRack,
}

impl StructureKind {
    const fn diameter(self) -> Length {
        Length::from_millimeters(match self {
            Self::Campfire | Self::Fireplace | Self::FishTrap => 800,
            Self::LeanTo => 2_500,
            Self::Bed => 2_000,
            Self::Cabin => 4_000,
            Self::Snare | Self::SprungSnare => 300,
            Self::DryingRack => 1_500,
        })
    }

    /// Nothing built is counted as tall enough to hide what is behind it, since most of
    /// what is built is used from under or inside it.
    const fn height(self) -> Length {
        Length::from_millimeters(match self {
            Self::Campfire | Self::Fireplace | Self::Snare | Self::SprungSnare => 300,
            Self::Bed => 200,
            Self::FishTrap => 500,
            Self::LeanTo | Self::Cabin | Self::DryingRack => 1_500,
        })
    }

    /// Returns whether the structure keeps the rain and the wind off whoever is at it.
    pub const fn is_shelter(self) -> bool {
        matches!(self, Self::LeanTo | Self::Cabin)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DAY: u64 = SECONDS_PER_DAY;

    fn plant(species: PlantSpecies, diameter: i64) -> Plant {
        Plant::new(
            Object::default(),
            species,
            Length::from_meters(12),
            Length::from_millimeters(diameter),
        )
    }

    fn day(number: u64) -> Timestamp {
        Timestamp::from_seconds(number * DAY)
    }

    #[test]
    fn dead_wood_taken_from_a_tree_comes_back_slowly() {
        let mut tree = plant(PlantSpecies::Pine, 200);
        assert_eq!(tree.stock(day(400)), 10);

        tree.take(4, day(400));
        assert_eq!(tree.stock(day(400)), 6);
        assert_eq!(tree.stock(day(404)), 6);
        assert_eq!(tree.stock(day(405)), 7);

        tree.take(7, day(405));
        assert_eq!(tree.stock(day(405)), 0);
        assert_eq!(tree.stock(day(5_000)), 10);
    }

    #[test]
    fn berries_picked_are_gone_until_the_next_year() {
        let mut patch = plant(PlantSpecies::Bilberry, 2_000);
        assert_eq!(patch.stock(day(200)), 20);

        patch.take(15, day(200));
        patch.take(2, day(210));
        assert_eq!(patch.stock(day(250)), 3);
        assert_eq!(patch.stock(day(365 + 200)), 20);

        patch.take(50, day(365 + 200));
        assert_eq!(patch.stock(day(365 + 201)), 0);
    }

    #[test]
    fn berries_ripen_in_their_season() {
        let date = |ordinal: u64| Date::from_timestamp(day(ordinal - 1));

        assert!(!PlantSpecies::Bilberry.is_ripe(date(150)));
        assert!(PlantSpecies::Bilberry.is_ripe(date(200)));
        assert!(!PlantSpecies::Lingonberry.is_ripe(date(200)));
        assert!(PlantSpecies::Lingonberry.is_ripe(date(280)));
        assert!(!PlantSpecies::Bilberry.is_ripe(date(280)));
        assert!(!PlantSpecies::Birch.is_ripe(date(200)));
    }

    #[test]
    fn only_a_grown_tree_gives_logs() {
        assert_eq!(plant(PlantSpecies::Spruce, 80).timber(), (0, 6));
        assert_eq!(plant(PlantSpecies::Spruce, 320).timber(), (3, 12));
        assert!(!plant(PlantSpecies::Bilberry, 2_000).is_grown());
    }

    #[test]
    fn pile_looks_to_be_of_what_there_is_the_most_of_by_weight() {
        let pile = Pile::new(
            Object::default(),
            [Stack::new(Item::Stick, 10), Stack::new(Item::Log, 1)],
        );

        assert_eq!(pile.main_item(), Item::Log);
        assert_eq!(Thing::Pile(pile).kind(), Kind::Pile(Item::Log));
    }

    #[test]
    fn fire_is_stoked_from_now_once_it_is_out_and_only_so_far() {
        let mut fire = Structure::new(Object::default(), StructureKind::Fireplace);
        let now = Timestamp::from_seconds(1_000);

        fire.stoke(900, now);
        assert_eq!(fire.until(), Timestamp::from_seconds(1_900));

        fire.stoke(900, now);
        assert_eq!(fire.until(), Timestamp::from_seconds(2_800));

        assert_eq!(fire.room(now), Structure::LONGEST_BURN - 1_800);
        fire.stoke(1_000_000, now);
        assert_eq!(fire.room(now), 0);
        assert_eq!(
            fire.until(),
            Timestamp::from_seconds(1_000 + Structure::LONGEST_BURN)
        );
    }

    #[test]
    fn thing_written_before_plants_were_harvested_still_reads() {
        let old = "Plant((object:(location:(x:1,y:2),creation:0),species:Birch,\
                   height:15000,trunk_diameter:300))";

        let read: Thing = ron::from_str(old).expect("the old plant decodes");

        assert_eq!(read.kind(), Kind::Plant(PlantSpecies::Birch));
        assert_eq!(read.location(), Location::new(1, 2));
    }
}
