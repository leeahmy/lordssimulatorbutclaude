//! The better ways of doing things that a settlement works out, from smoking fish to
//! the internet, and the ages they belong to.

use serde::{Deserialize, Serialize};

/// A stretch of history, told by what is known in it.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Age {
    Stone,
    Bronze,
    Iron,
    Middle,
    Print,
    Steam,
    Electric,
    Modern,
}

impl Age {
    pub const ALL: [Self; 8] = [
        Self::Stone,
        Self::Bronze,
        Self::Iron,
        Self::Middle,
        Self::Print,
        Self::Steam,
        Self::Electric,
        Self::Modern,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Stone => "Stone Age",
            Self::Bronze => "Bronze Age",
            Self::Iron => "Iron Age",
            Self::Middle => "Middle Ages",
            Self::Print => "Age of Print",
            Self::Steam => "Age of Steam",
            Self::Electric => "Age of Electricity",
            Self::Modern => "Modern Age",
        }
    }

    /// Returns how many times what is asked of the thinkers of the first age working out
    /// an advance of the age asks.
    pub const fn effort(self) -> u32 {
        match self {
            Self::Stone => 15,
            Self::Bronze => 80,
            Self::Iron => 110,
            Self::Middle => 200,
            Self::Print => 1_000,
            Self::Steam => 1_100,
            Self::Electric => 1_300,
            Self::Modern => 1_800,
        }
    }

    /// Returns the age of a settlement that knows `advances`: the latest age of which it
    /// knows at least two advances, or the first.
    pub fn of(advances: &[Advance]) -> Self {
        let known = |age: Self| {
            advances
                .iter()
                .filter(|advance| advance.age() == age)
                .count()
        };
        let reached = Self::ALL.into_iter().rev().find(|age| known(*age) >= 2);
        reached.unwrap_or(Self::Stone)
    }
}

/// A better way of doing things that a settlement can work out.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Advance {
    Smoking,
    Pottery,
    Boats,
    Herbalism,
    Weaving,
    Farming,
    Masonry,
    Ironworking,
    Writing,
    Husbandry,
    Brewing,
    Mining,
    Charcoal,
    Bronze,
    Wheel,
    Plough,
    Arithmetic,
    Coinage,
    Law,
    Medicine,
    Sailing,
    Teaching,
    Mills,
    CropRotation,
    Brickmaking,
    Glassmaking,
    Guilds,
    Papermaking,
    Steel,
    Banking,
    Printing,
    Optics,
    Navigation,
    Science,
    Sanitation,
    Vaccination,
    SteamPower,
    Mechanization,
    Railways,
    Chemistry,
    Agronomy,
    Concrete,
    Canning,
    Electricity,
    Telegraph,
    Combustion,
    Refrigeration,
    Radio,
    Antibiotics,
    Electronics,
    Aviation,
    Television,
    Computers,
    ModernMedicine,
    Internet,
    CleanPower,
}

/// Everything there is to tell about an advance.
struct Spec {
    name: &'static str,
    age: Age,
    cost: u32,
    requires: &'static [Advance],
    use_of: &'static str,
}

impl Advance {
    pub const ALL: [Self; 56] = [
        Self::Smoking,
        Self::Pottery,
        Self::Boats,
        Self::Herbalism,
        Self::Weaving,
        Self::Farming,
        Self::Masonry,
        Self::Ironworking,
        Self::Writing,
        Self::Husbandry,
        Self::Brewing,
        Self::Mining,
        Self::Charcoal,
        Self::Bronze,
        Self::Wheel,
        Self::Plough,
        Self::Arithmetic,
        Self::Coinage,
        Self::Law,
        Self::Medicine,
        Self::Sailing,
        Self::Teaching,
        Self::Mills,
        Self::CropRotation,
        Self::Brickmaking,
        Self::Glassmaking,
        Self::Guilds,
        Self::Papermaking,
        Self::Steel,
        Self::Banking,
        Self::Printing,
        Self::Optics,
        Self::Navigation,
        Self::Science,
        Self::Sanitation,
        Self::Vaccination,
        Self::SteamPower,
        Self::Mechanization,
        Self::Railways,
        Self::Chemistry,
        Self::Agronomy,
        Self::Concrete,
        Self::Canning,
        Self::Electricity,
        Self::Telegraph,
        Self::Combustion,
        Self::Refrigeration,
        Self::Radio,
        Self::Antibiotics,
        Self::Electronics,
        Self::Aviation,
        Self::Television,
        Self::Computers,
        Self::ModernMedicine,
        Self::Internet,
        Self::CleanPower,
    ];

    const fn spec(self) -> Spec {
        match self {
            Self::Smoking => Spec {
                name: "Smoking",
                age: Age::Stone,
                cost: 15,
                requires: &[],
                use_of: "Smokehouses can be built, with which catches are kept for good",
            },
            Self::Pottery => Spec {
                name: "Pottery",
                age: Age::Stone,
                cost: 25,
                requires: &[],
                use_of: "Clay can be fired in a kiln into pots, and food in the store keeps twice as long in jars",
            },
            Self::Boats => Spec {
                name: "Boats",
                age: Age::Stone,
                cost: 30,
                requires: &[],
                use_of: "Docks can be built, from which fishers go out and catch twice as much",
            },
            Self::Herbalism => Spec {
                name: "Herbalism",
                age: Age::Stone,
                cost: 30,
                requires: &[],
                use_of: "Far fewer die of sickness, and medicine can be boiled from berries and bark",
            },
            Self::Weaving => Spec {
                name: "Weaving",
                age: Age::Stone,
                cost: 35,
                requires: &[],
                use_of: "Everyone is clothed for the winter, pelts or no pelts, and weaving sheds can be built",
            },
            Self::Farming => Spec {
                name: "Farming",
                age: Age::Stone,
                cost: 40,
                requires: &[],
                use_of: "Fields can be cleared, on which farmers grow rye",
            },
            Self::Masonry => Spec {
                name: "Masonry",
                age: Age::Stone,
                cost: 50,
                requires: &[Self::Pottery],
                use_of: "Wells, quarries, and bathhouses can be built",
            },
            Self::Ironworking => Spec {
                name: "Ironworking",
                age: Age::Iron,
                cost: 140,
                requires: &[Self::Bronze],
                use_of: "Bog ore can be raked from the lake and iron ore mined, and iron tools make all work yield a quarter more",
            },
            Self::Writing => Spec {
                name: "Writing",
                age: Age::Bronze,
                cost: 100,
                requires: &[Self::Pottery],
                use_of: "Thinkers build on what was written down, and get half again as far",
            },
            Self::Husbandry => Spec {
                name: "Animal husbandry",
                age: Age::Stone,
                cost: 45,
                requires: &[Self::Farming],
                use_of: "Pastures can be fenced, on which herders keep animals for meat, cheese, wool, and hides",
            },
            Self::Brewing => Spec {
                name: "Brewing",
                age: Age::Stone,
                cost: 35,
                requires: &[Self::Farming, Self::Pottery],
                use_of: "Breweries and taverns can be built. Where there is ale to drink, fewer sicken from the water",
            },
            Self::Mining => Spec {
                name: "Mining",
                age: Age::Bronze,
                cost: 70,
                requires: &[Self::Masonry],
                use_of: "Mines can be dug, from which miners bring copper and tin ore",
            },
            Self::Charcoal => Spec {
                name: "Charcoal burning",
                age: Age::Bronze,
                cost: 50,
                requires: &[Self::Pottery],
                use_of: "Charcoal kilns can be built. Charcoal burns hot enough to smelt ore",
            },
            Self::Bronze => Spec {
                name: "Bronze working",
                age: Age::Bronze,
                cost: 90,
                requires: &[Self::Mining, Self::Charcoal],
                use_of: "Smithies can be built to cast bronze in, and bronze tools make all work yield a sixth more",
            },
            Self::Wheel => Spec {
                name: "The wheel",
                age: Age::Bronze,
                cost: 60,
                requires: &[Self::Farming],
                use_of: "With carts, woodcutters, gatherers, and quarriers bring in half again as much",
            },
            Self::Plough => Spec {
                name: "The plough",
                age: Age::Bronze,
                cost: 70,
                requires: &[Self::Farming, Self::Husbandry],
                use_of: "Farmers grow a quarter more",
            },
            Self::Arithmetic => Spec {
                name: "Arithmetic",
                age: Age::Bronze,
                cost: 80,
                requires: &[Self::Writing],
                use_of: "What is known can be reckoned with: thinkers get a tenth further",
            },
            Self::Coinage => Spec {
                name: "Coinage",
                age: Age::Iron,
                cost: 120,
                requires: &[Self::Bronze, Self::Arithmetic],
                use_of: "Markets can be built. Goods are bought and sold for coin, work is paid, and taxes are raised",
            },
            Self::Law => Spec {
                name: "Written law",
                age: Age::Iron,
                cost: 140,
                requires: &[Self::Writing],
                use_of: "Courthouses can be built, and a town with a town hall can be chartered as a city",
            },
            Self::Medicine => Spec {
                name: "Medicine",
                age: Age::Iron,
                cost: 140,
                requires: &[Self::Herbalism, Self::Writing],
                use_of: "Infirmaries can be built, where healers tend the sick",
            },
            Self::Sailing => Spec {
                name: "Sailing",
                age: Age::Iron,
                cost: 120,
                requires: &[Self::Boats, Self::Weaving],
                use_of: "Harbours can be built, to which trading ships come",
            },
            Self::Teaching => Spec {
                name: "Teaching",
                age: Age::Iron,
                cost: 110,
                requires: &[Self::Writing],
                use_of: "Schools can be built, where children learn to read. The more who read, the further thinkers get",
            },
            Self::Mills => Spec {
                name: "Water mills",
                age: Age::Middle,
                cost: 180,
                requires: &[Self::Wheel, Self::Masonry],
                use_of: "Mills grind a tenth more out of every harvest, and sawmills cut logs into planks",
            },
            Self::CropRotation => Spec {
                name: "Crop rotation",
                age: Age::Middle,
                cost: 180,
                requires: &[Self::Plough],
                use_of: "Fields rest in turn and are sown with turnips: farmers grow a fifth more",
            },
            Self::Brickmaking => Spec {
                name: "Brickmaking",
                age: Age::Middle,
                cost: 150,
                requires: &[Self::Pottery, Self::Masonry],
                use_of: "Brickworks can be built, and bricks fired in kilns. Town houses are built of brick",
            },
            Self::Glassmaking => Spec {
                name: "Glassmaking",
                age: Age::Middle,
                cost: 180,
                requires: &[Self::Charcoal, Self::Pottery],
                use_of: "Glassworks can be built, where sand is melted into panes",
            },
            Self::Guilds => Spec {
                name: "Guilds",
                age: Age::Middle,
                cost: 200,
                requires: &[Self::Coinage, Self::Law],
                use_of: "A guildhall can be built, under which every craft yields a quarter more",
            },
            Self::Papermaking => Spec {
                name: "Papermaking",
                age: Age::Middle,
                cost: 180,
                requires: &[Self::Writing, Self::Mills],
                use_of: "Sawmills make paper as well, and libraries can be built",
            },
            Self::Steel => Spec {
                name: "Steel",
                age: Age::Middle,
                cost: 260,
                requires: &[Self::Ironworking, Self::Charcoal, Self::Mills],
                use_of: "Steelworks can be built, coal is mined, and steel tools make all work yield two fifths more",
            },
            Self::Banking => Spec {
                name: "Banking",
                age: Age::Middle,
                cost: 240,
                requires: &[Self::Guilds, Self::Arithmetic],
                use_of: "A bank can be built, at which the treasury earns interest",
            },
            Self::Printing => Spec {
                name: "Printing",
                age: Age::Print,
                cost: 320,
                requires: &[Self::Papermaking, Self::Bronze],
                use_of: "Printing houses can be built. Books are printed, and what you stand for can be put in a newspaper",
            },
            Self::Optics => Spec {
                name: "Optics",
                age: Age::Print,
                cost: 300,
                requires: &[Self::Glassmaking, Self::Arithmetic],
                use_of: "With lenses to see by, thinkers get a fifth further",
            },
            Self::Navigation => Spec {
                name: "Navigation",
                age: Age::Print,
                cost: 320,
                requires: &[Self::Sailing, Self::Optics],
                use_of: "Ships sail farther: harbours trade twice as much, and potatoes are brought home to grow",
            },
            Self::Science => Spec {
                name: "The scientific method",
                age: Age::Print,
                cost: 500,
                requires: &[Self::Printing, Self::Optics],
                use_of: "Universities can be built, whose scholars get three times as far as thinkers",
            },
            Self::Sanitation => Spec {
                name: "Sanitation",
                age: Age::Print,
                cost: 380,
                requires: &[Self::Medicine, Self::Brickmaking],
                use_of: "Waterworks can be built, with which crowded towns sicken half as much",
            },
            Self::Vaccination => Spec {
                name: "Vaccination",
                age: Age::Print,
                cost: 450,
                requires: &[Self::Medicine, Self::Science],
                use_of: "Far fewer children die",
            },
            Self::SteamPower => Spec {
                name: "Steam power",
                age: Age::Steam,
                cost: 650,
                requires: &[Self::Steel, Self::Science],
                use_of: "Factories can be built. Mines and mills driven by steam yield twice as much",
            },
            Self::Mechanization => Spec {
                name: "Mechanized weaving",
                age: Age::Steam,
                cost: 600,
                requires: &[Self::SteamPower, Self::Weaving],
                use_of: "Textile mills can be built",
            },
            Self::Railways => Spec {
                name: "Railways",
                age: Age::Steam,
                cost: 750,
                requires: &[Self::SteamPower],
                use_of: "A railway station can be built, which doubles trade and brings twice as many to settle",
            },
            Self::Chemistry => Spec {
                name: "Chemistry",
                age: Age::Steam,
                cost: 650,
                requires: &[Self::Science, Self::Glassmaking],
                use_of: "Medicine that works is made in quantity: healers save twice as many",
            },
            Self::Agronomy => Spec {
                name: "Agronomy",
                age: Age::Steam,
                cost: 600,
                requires: &[Self::CropRotation, Self::Chemistry],
                use_of: "With fertilizer and better seed, farmers grow twice as much",
            },
            Self::Concrete => Spec {
                name: "Concrete",
                age: Age::Steam,
                cost: 500,
                requires: &[Self::Brickmaking, Self::Chemistry],
                use_of: "Brickworks burn cement as well, of which the largest buildings are made",
            },
            Self::Canning => Spec {
                name: "Canning",
                age: Age::Steam,
                cost: 500,
                requires: &[Self::Steel, Self::Chemistry],
                use_of: "Canneries can be built, which put up food that never goes off",
            },
            Self::Electricity => Spec {
                name: "Electricity",
                age: Age::Electric,
                cost: 1_000,
                requires: &[Self::SteamPower, Self::Chemistry],
                use_of: "Power stations can be built. With power, factories and mills yield half again as much",
            },
            Self::Telegraph => Spec {
                name: "The telegraph",
                age: Age::Electric,
                cost: 800,
                requires: &[Self::Electricity],
                use_of: "Telegraph offices can be built: with news of prices, trade pays a fifth more",
            },
            Self::Combustion => Spec {
                name: "The combustion engine",
                age: Age::Electric,
                cost: 1_100,
                requires: &[Self::SteamPower, Self::Chemistry],
                use_of: "Machine depots can be built. With tractors and oil, farmers grow three times as much",
            },
            Self::Refrigeration => Spec {
                name: "Refrigeration",
                age: Age::Electric,
                cost: 900,
                requires: &[Self::Electricity],
                use_of: "Cold stores can be built, in which food does not go off",
            },
            Self::Radio => Spec {
                name: "Radio",
                age: Age::Electric,
                cost: 1_100,
                requires: &[Self::Telegraph],
                use_of: "A broadcasting house can be built, from which you can speak to everyone at once",
            },
            Self::Antibiotics => Spec {
                name: "Antibiotics",
                age: Age::Electric,
                cost: 1_200,
                requires: &[Self::Vaccination, Self::Chemistry],
                use_of: "Hospitals can be built, and sickness kills a third as many",
            },
            Self::Electronics => Spec {
                name: "Electronics",
                age: Age::Modern,
                cost: 1_500,
                requires: &[Self::Radio],
                use_of: "Research institutes can be built, and factories make goods worth far more",
            },
            Self::Aviation => Spec {
                name: "Aviation",
                age: Age::Modern,
                cost: 1_500,
                requires: &[Self::Combustion],
                use_of: "An airport can be built, which brings trade and people from everywhere",
            },
            Self::Television => Spec {
                name: "Television",
                age: Age::Modern,
                cost: 1_400,
                requires: &[Self::Electronics],
                use_of: "What you broadcast is seen as well as heard, and wins over twice as many",
            },
            Self::Computers => Spec {
                name: "Computers",
                age: Age::Modern,
                cost: 1_800,
                requires: &[Self::Electronics],
                use_of: "Computing centres can be built, with which scholars get twice as far",
            },
            Self::ModernMedicine => Spec {
                name: "Modern medicine",
                age: Age::Modern,
                cost: 1_800,
                requires: &[Self::Antibiotics, Self::Electronics],
                use_of: "People live ten years longer",
            },
            Self::Internet => Spec {
                name: "The internet",
                age: Age::Modern,
                cost: 2_200,
                requires: &[Self::Computers, Self::Telegraph],
                use_of: "Everything known is at hand: scholars get half again as far, and trade pays a fifth more",
            },
            Self::CleanPower => Spec {
                name: "Clean power",
                age: Age::Modern,
                cost: 2_000,
                requires: &[Self::Electricity, Self::Electronics],
                use_of: "Power stations run on wind, water, and sun, and burn no coal",
            },
        }
    }

    pub const fn name(self) -> &'static str {
        self.spec().name
    }

    /// Returns the age the advance belongs to.
    pub const fn age(self) -> Age {
        self.spec().age
    }

    /// Returns what the advance is good for.
    pub const fn use_of(self) -> &'static str {
        self.spec().use_of
    }

    /// Returns how many days of one thinker's work the advance takes. Later ages take
    /// far more of it, as there are far more who think, with far more to help them.
    pub const fn cost(self) -> u32 {
        let spec = self.spec();
        spec.cost * spec.age.effort()
    }

    /// Returns the advances that have to be known first.
    pub const fn requires(self) -> &'static [Self] {
        self.spec().requires
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_advance_is_listed_once_and_says_what_it_is_good_for() {
        for (index, advance) in Advance::ALL.into_iter().enumerate() {
            assert!(!Advance::ALL[..index].contains(&advance), "{advance:?}");
            assert!(advance.cost() > 0, "{advance:?}");
            assert!(!advance.name().is_empty() && !advance.use_of().is_empty());
            assert!(!advance.use_of().ends_with('.'), "{advance:?}");
        }
    }

    #[test]
    fn advances_take_only_what_is_of_their_age_or_an_earlier_one() {
        for advance in Advance::ALL {
            for needed in advance.requires() {
                assert!(
                    needed.age() <= advance.age(),
                    "{advance:?} takes {needed:?}"
                );
                assert_ne!(*needed, advance);
            }
        }
    }

    #[test]
    fn every_advance_can_be_reached_and_later_ages_cost_more() {
        let mut known: Vec<Advance> = Vec::new();
        while known.len() < Advance::ALL.len() {
            let next = Advance::ALL.into_iter().find(|advance| {
                !known.contains(advance)
                    && advance
                        .requires()
                        .iter()
                        .all(|needed| known.contains(needed))
            });
            known.push(next.expect("something can always be worked out until all is known"));
        }

        let cost = |age: Age| -> u32 {
            let of_age = Advance::ALL
                .into_iter()
                .filter(|advance| advance.age() == age);
            of_age.map(Advance::cost).sum()
        };
        for ages in Age::ALL.windows(2) {
            assert!(cost(ages[0]) < cost(ages[1]), "{ages:?}");
        }
    }

    #[test]
    fn age_is_told_by_what_is_known() {
        assert_eq!(Age::of(&[]), Age::Stone);
        assert_eq!(Age::of(&[Advance::Pottery, Advance::Mining]), Age::Stone);
        assert_eq!(
            Age::of(&[Advance::Pottery, Advance::Mining, Advance::Bronze]),
            Age::Bronze
        );
        assert_eq!(Age::of(&Advance::ALL), Age::Modern);
        assert_eq!(Age::Steam.name(), "Age of Steam");
    }
}
