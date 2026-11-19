//! What the people of a settlement hold on how to live together, who they follow, and the
//! laws they live by.

use serde::{Deserialize, Serialize};

use crate::advance::Advance;
use crate::rng::Rng;

/// Something people hold different views on.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Issue {
    /// Whether what is made belongs to all or to the one who made it.
    Property,
    /// Whether to keep to the old ways or to try new ones.
    Change,
    /// Whether the elders decide or everyone has a say.
    Rule,
    /// Whether strangers are kept out or made welcome.
    Strangers,
}

impl Issue {
    pub const ALL: [Self; 4] = [Self::Property, Self::Change, Self::Rule, Self::Strangers];

    pub fn name(self) -> &'static str {
        match self {
            Self::Property => "Property",
            Self::Change => "Change",
            Self::Rule => "Rule",
            Self::Strangers => "Strangers",
        }
    }

    /// Returns what those who lean to the one side and to the other of the issue hold.
    pub fn sides(self) -> (&'static str, &'static str) {
        match self {
            Self::Property => ("All is shared", "Each to their own"),
            Self::Change => ("The old ways", "New ways"),
            Self::Rule => ("The elders decide", "Everyone has a say"),
            Self::Strangers => ("Wary of strangers", "Strangers are welcome"),
        }
    }
}

/// How far to which side of each issue someone leans, from -2 to 2, in the order of
/// [`Issue::ALL`].
pub type Views = [i8; 4];

/// The farthest anyone leans to either side of an issue.
pub const FIRMEST: i8 = 2;

/// Returns how much alike `own` and `their` views are: 8 for the very same, and -8 for
/// the very opposite.
pub fn agreement(own: &Views, their: &Views) -> i32 {
    own.iter()
        .zip(their)
        .map(|(own, their)| 2 - i32::from((own - their).abs()))
        .sum()
}

/// Returns views that nobody chose.
pub(crate) fn random_views(rng: &mut Rng) -> Views {
    [(); 4].map(|()| rng.between((-2, 2)) as i8)
}

/// Returns the views of a child of parents of the views given, which are near those of
/// the parents.
pub(crate) fn inherited_views(rng: &mut Rng, parents: [&Views; 2]) -> Views {
    let mut views = [0; 4];
    for (issue, view) in views.iter_mut().enumerate() {
        let mean = (i64::from(parents[0][issue]) + i64::from(parents[1][issue])) / 2;
        *view = (mean + rng.between((-1, 1))).clamp(-2, 2) as i8;
    }
    views
}

/// A rule a settlement lives by.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Law {
    SharedTable,
    OwnHearths,
    OldCustoms,
    Schooling,
    EldersWord,
    OpenMoot,
    ClosedGates,
    OpenGates,
    PublicWorks,
    FreeMarket,
    GuildRights,
    FreeEnquiry,
    Birthright,
    Assembly,
    Tariffs,
    FreePort,
}

impl Law {
    pub const ALL: [Self; 16] = [
        Self::SharedTable,
        Self::OwnHearths,
        Self::OldCustoms,
        Self::Schooling,
        Self::EldersWord,
        Self::OpenMoot,
        Self::ClosedGates,
        Self::OpenGates,
        Self::PublicWorks,
        Self::FreeMarket,
        Self::GuildRights,
        Self::FreeEnquiry,
        Self::Birthright,
        Self::Assembly,
        Self::Tariffs,
        Self::FreePort,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::SharedTable => "The shared table",
            Self::OwnHearths => "Hearths of their own",
            Self::OldCustoms => "The old customs",
            Self::Schooling => "Schooling",
            Self::EldersWord => "The elder's word",
            Self::OpenMoot => "The open moot",
            Self::ClosedGates => "Closed gates",
            Self::OpenGates => "Open gates",
            Self::PublicWorks => "Public works",
            Self::FreeMarket => "The free market",
            Self::GuildRights => "Guild rights",
            Self::FreeEnquiry => "Free enquiry",
            Self::Birthright => "Birthright",
            Self::Assembly => "The assembly",
            Self::Tariffs => "Tariffs",
            Self::FreePort => "The free port",
        }
    }

    /// Returns what living by the law comes to.
    pub fn effect(self) -> &'static str {
        match self {
            Self::SharedTable => {
                "Everyone eats at one table: a tenth less food is needed, and a little less is brought in"
            }
            Self::OwnHearths => {
                "Each household keeps part of what it brings in: more food is brought in, and more is eaten"
            }
            Self::OldCustoms => {
                "People bear hunger longer before they leave, and thinkers get less far"
            }
            Self::Schooling => {
                "Children learn until fourteen before they work, and thinkers get half again as far"
            }
            Self::EldersWord => "Whoever leads does so for life, and makes the laws without asking",
            Self::OpenMoot => {
                "A leader is chosen every year, and every law is put to all the grown-ups"
            }
            Self::ClosedGates => {
                "Nobody is let in to settle, and fewer sicknesses come from outside"
            }
            Self::OpenGates => {
                "Twice as many come to settle, and they are let in though houses are short"
            }
            Self::PublicWorks => {
                "What is built costs the treasury no coin, and taxes are never less than a fifth"
            }
            Self::FreeMarket => {
                "Half again as much is traded, and taxes are never more than a tenth"
            }
            Self::GuildRights => {
                "Every craft yields a tenth more, and thinkers and scholars get a tenth less far"
            }
            Self::FreeEnquiry => {
                "Scholars get a quarter further, whatever the old may think of what they find"
            }
            Self::Birthright => {
                "When whoever leads dies, their eldest grown child leads after them, and no moot is held for it"
            }
            Self::Assembly => {
                "A leader is chosen every two years, and no law is made without two thirds for it"
            }
            Self::Tariffs => {
                "What is bought from outside costs a quarter more, and a tenth of what is sold goes to the treasury on top"
            }
            Self::FreePort => "Half again as much is traded, and half again as many come to settle",
        }
    }

    pub const fn issue(self) -> Issue {
        match self {
            Self::SharedTable => Issue::Property,
            Self::OwnHearths => Issue::Property,
            Self::OldCustoms => Issue::Change,
            Self::Schooling => Issue::Change,
            Self::EldersWord => Issue::Rule,
            Self::OpenMoot => Issue::Rule,
            Self::ClosedGates => Issue::Strangers,
            Self::OpenGates => Issue::Strangers,
            Self::PublicWorks => Issue::Property,
            Self::FreeMarket => Issue::Property,
            Self::GuildRights => Issue::Change,
            Self::FreeEnquiry => Issue::Change,
            Self::Birthright => Issue::Rule,
            Self::Assembly => Issue::Rule,
            Self::Tariffs => Issue::Strangers,
            Self::FreePort => Issue::Strangers,
        }
    }

    /// Returns which side of its issue the law is on: -1 or 1.
    pub const fn side(self) -> i8 {
        match self {
            Self::SharedTable => -1,
            Self::OwnHearths => 1,
            Self::OldCustoms => -1,
            Self::Schooling => 1,
            Self::EldersWord => -1,
            Self::OpenMoot => 1,
            Self::ClosedGates => -1,
            Self::OpenGates => 1,
            Self::PublicWorks => -1,
            Self::FreeMarket => 1,
            Self::GuildRights => -1,
            Self::FreeEnquiry => 1,
            Self::Birthright => -1,
            Self::Assembly => 1,
            Self::Tariffs => -1,
            Self::FreePort => 1,
        }
    }

    /// Returns the law that cannot be in force at the same time.
    pub const fn opposite(self) -> Self {
        match self {
            Self::SharedTable => Self::OwnHearths,
            Self::OwnHearths => Self::SharedTable,
            Self::OldCustoms => Self::Schooling,
            Self::Schooling => Self::OldCustoms,
            Self::EldersWord => Self::OpenMoot,
            Self::OpenMoot => Self::EldersWord,
            Self::ClosedGates => Self::OpenGates,
            Self::OpenGates => Self::ClosedGates,
            Self::PublicWorks => Self::FreeMarket,
            Self::FreeMarket => Self::PublicWorks,
            Self::GuildRights => Self::FreeEnquiry,
            Self::FreeEnquiry => Self::GuildRights,
            Self::Birthright => Self::Assembly,
            Self::Assembly => Self::Birthright,
            Self::Tariffs => Self::FreePort,
            Self::FreePort => Self::Tariffs,
        }
    }

    /// Returns what a settlement has to know before the law can be thought of.
    pub const fn needs(self) -> Option<Advance> {
        match self {
            Self::PublicWorks => Some(Advance::Coinage),
            Self::FreeMarket => Some(Advance::Coinage),
            Self::GuildRights => Some(Advance::Guilds),
            Self::FreeEnquiry => Some(Advance::Printing),
            Self::Assembly => Some(Advance::Law),
            Self::Tariffs => Some(Advance::Coinage),
            Self::FreePort => Some(Advance::Sailing),
            _ => None,
        }
    }
}

/// What a settlement has that its ways of deciding things are told by.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Civics {
    pub people: u32,
    pub writing: bool,
    /// Whether there is a meeting hall or a town hall.
    pub hall: bool,
    /// Whether there are written law and a town hall to be chartered with.
    pub charter: bool,
    /// Whether there is a power station.
    pub power: bool,
}

/// How far a settlement has come, by which its ways of deciding things are told.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Era {
    /// A handful of people who settle things among themselves.
    Band,
    /// Enough people that laws are put to a vote.
    Village,
    /// A place with a hall and with writing, which holds elections.
    Town,
    /// A chartered town of many people.
    City,
    /// A great city lit by electricity.
    Metropolis,
}

impl Era {
    /// The fewest people a village, a town, a city, and a metropolis have.
    pub const VILLAGE: u32 = 8;
    pub const TOWN: u32 = 25;
    pub const CITY: u32 = 120;
    pub const METROPOLIS: u32 = 500;

    /// Returns the era of a settlement that has what `civics` tells of.
    pub const fn of(civics: Civics) -> Self {
        let town = civics.people >= Self::TOWN && civics.writing && civics.hall;
        let city = town && civics.people >= Self::CITY && civics.charter;

        if city && civics.people >= Self::METROPOLIS && civics.power {
            Self::Metropolis
        } else if city {
            Self::City
        } else if town {
            Self::Town
        } else if civics.people >= Self::VILLAGE {
            Self::Village
        } else {
            Self::Band
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Band => "Band",
            Self::Village => "Village",
            Self::Town => "Town",
            Self::City => "City",
            Self::Metropolis => "Metropolis",
        }
    }

    /// Returns how things are decided in the era.
    pub fn ways(self) -> &'static str {
        match self {
            Self::Band => {
                "A leader is chosen at the spring moot each year, and makes the laws alone. \
                 With eight people it becomes a village."
            }
            Self::Village => {
                "A leader is chosen at the spring moot each year. Laws are put to all the \
                 grown-ups. With twenty-five people, a meeting hall, and writing it becomes \
                 a town."
            }
            Self::Town => {
                "A leader is elected every four years, after a month of campaigning. Laws \
                 are put to all the grown-ups. With a hundred and twenty people, written \
                 law, and a town hall it is chartered as a city."
            }
            Self::City => {
                "A mayor is elected every four years, after a month of campaigning. Laws \
                 are put to all the grown-ups. With five hundred people and a power \
                 station it becomes a metropolis."
            }
            Self::Metropolis => {
                "A mayor is elected every four years, after a month of campaigning. Laws \
                 are put to all the grown-ups."
            }
        }
    }

    /// Returns what whoever leads is called.
    pub fn title(self) -> &'static str {
        match self {
            Self::Band | Self::Village => "leader",
            Self::Town => "headman",
            Self::City | Self::Metropolis => "mayor",
        }
    }

    /// Returns how many years pass between the choosing of leaders.
    pub const fn term(self) -> u64 {
        match self {
            Self::Band | Self::Village => 1,
            Self::Town | Self::City | Self::Metropolis => 4,
        }
    }

    /// Returns whether leaders are chosen after a month of campaigning.
    pub const fn campaigns(self) -> bool {
        matches!(self, Self::Town | Self::City | Self::Metropolis)
    }
}

/// A charge that whoever leads gives to someone they trust with it.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Office {
    /// Sees to the defence.
    Marshal,
    /// Sees to the taxes.
    Treasurer,
    /// Sees to those who think.
    Chancellor,
    /// Sees to those in need.
    Almoner,
}

impl Office {
    pub const ALL: [Self; 4] = [
        Self::Marshal,
        Self::Treasurer,
        Self::Chancellor,
        Self::Almoner,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Marshal => "Marshal",
            Self::Treasurer => "Treasurer",
            Self::Chancellor => "Chancellor",
            Self::Almoner => "Almoner",
        }
    }

    /// Returns what comes of the office being held, and by whom it is held best.
    pub fn telling(self) -> &'static str {
        match self {
            Self::Marshal => {
                "Sees to the defence, which is worth two more, or four under someone bold"
            }
            Self::Treasurer => {
                "Sees to the taxes, of which a tenth more is had, or a fifth under someone \
                 stern"
            }
            Self::Chancellor => {
                "Sees to those who think, who get a twentieth further, or a tenth under \
                 someone clever"
            }
            Self::Almoner => {
                "Sees to those in need, which makes people more content, and more so under \
                 someone kind"
            }
        }
    }

    /// Returns the trait that makes for the best holder of the office.
    pub const fn suits(self) -> crate::people::Trait {
        use crate::people::Trait;
        match self {
            Self::Marshal => Trait::Bold,
            Self::Treasurer => Trait::Stern,
            Self::Chancellor => Trait::Clever,
            Self::Almoner => Trait::Kind,
        }
    }
}

/// A way of winning people over.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Appeal {
    /// Speaking to everyone by the cairn.
    Speech,
    /// Feeding everyone out of one's own food.
    Feast,
    /// Putting up written notices of what one stands for.
    Notices,
    /// Taking space in the newspaper.
    Newspaper,
    /// Speaking over the air.
    Broadcast,
}

impl Appeal {
    pub const ALL: [Self; 5] = [
        Self::Speech,
        Self::Feast,
        Self::Notices,
        Self::Newspaper,
        Self::Broadcast,
    ];

    /// How many people hear a speech, and how many a feast is laid for, at the most.
    pub const CROWD: u32 = 60;
    pub const TABLE: u32 = 40;

    pub fn name(self) -> &'static str {
        match self {
            Self::Speech => "Speak to the people",
            Self::Feast => "Hold a feast",
            Self::Notices => "Put up notices",
            Self::Newspaper => "Take space in the newspaper",
            Self::Broadcast => "Speak over the air",
        }
    }

    pub fn effect(self) -> &'static str {
        match self {
            Self::Speech => {
                "An hour by the cairn, once a day. Wins over those who think like you, and \
                 puts off those who think the opposite. No more than sixty hear you."
            }
            Self::Feast => {
                "Four hours, once a month, out of food you carry or have piled up by the \
                 cairn: 1,500 kcal for each of your people, up to forty of them. Those who \
                 eat think better of you."
            }
            Self::Notices => {
                "Two hours and five pieces of birch bark or sheets of paper, once a week, \
                 once writing is known. Every grown-up who can read thinks a little better \
                 of you."
            }
            Self::Newspaper => {
                "An hour and 30 coins, once a week, where there is a printing house. \
                 Everyone who can read thinks better of you, those who think like you the \
                 most."
            }
            Self::Broadcast => {
                "An hour and 100 coins, once a week, where there is a broadcasting house. \
                 Every grown-up thinks better of you, and twice as much once there is \
                 television."
            }
        }
    }

    /// Returns how many in-game seconds it takes.
    pub const fn seconds(self) -> u32 {
        match self {
            Self::Speech | Self::Newspaper | Self::Broadcast => 3_600,
            Self::Feast => 4 * 3_600,
            Self::Notices => 2 * 3_600,
        }
    }

    /// Returns how many days pass before it can be done again.
    pub const fn rest_days(self) -> u64 {
        match self {
            Self::Speech => 1,
            Self::Feast => 30,
            Self::Notices | Self::Newspaper | Self::Broadcast => 7,
        }
    }

    /// Returns how many coins it costs.
    pub const fn coins(self) -> i64 {
        match self {
            Self::Speech | Self::Feast | Self::Notices => 0,
            Self::Newspaper => 30,
            Self::Broadcast => 100,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn views_agree_the_more_the_nearer_they_are() {
        let firm = [2, -2, 2, -2];
        let opposite = [-2, 2, -2, 2];
        let mild = [1, -1, 1, -1];

        assert_eq!(agreement(&firm, &firm), 8);
        assert_eq!(agreement(&firm, &opposite), -8);
        assert_eq!(agreement(&firm, &mild), 4);
        assert_eq!(agreement(&[0; 4], &firm), 0);
    }

    #[test]
    fn views_are_within_bounds_and_children_take_after_their_parents() {
        let mut rng = Rng::new(3);

        for _ in 0..500 {
            let views = random_views(&mut rng);
            assert!(views.iter().all(|view| (-FIRMEST..=FIRMEST).contains(view)));

            let child = inherited_views(&mut rng, [&[2; 4], &[2; 4]]);
            assert!(child.iter().all(|view| (1..=2).contains(view)));
        }
    }

    #[test]
    fn laws_come_in_pairs_on_opposite_sides_of_an_issue() {
        for law in Law::ALL {
            let opposite = law.opposite();

            assert_ne!(law, opposite);
            assert_eq!(opposite.opposite(), law);
            assert_eq!(law.issue(), opposite.issue());
            assert_eq!(law.side(), -opposite.side());
            assert!(!law.name().is_empty() && !law.effect().is_empty());
        }
        for issue in Issue::ALL {
            assert_eq!(
                Law::ALL.iter().filter(|law| law.issue() == issue).count(),
                4
            );
            assert_eq!(Issue::ALL[issue as usize], issue);
        }
    }

    #[test]
    fn settlement_matures_with_people_and_what_they_have_built_and_learned() {
        let civics = |people, writing, hall| Civics {
            people,
            writing,
            hall,
            ..Civics::default()
        };
        assert_eq!(Era::of(civics(7, true, true)), Era::Band);
        assert_eq!(Era::of(civics(8, false, false)), Era::Village);
        assert_eq!(Era::of(civics(60, true, false)), Era::Village);
        assert_eq!(Era::of(civics(60, false, true)), Era::Village);
        assert_eq!(Era::of(civics(25, true, true)), Era::Town);
        assert_eq!(Era::of(civics(400, true, true)), Era::Town);

        let chartered = |people, power| Civics {
            charter: true,
            power,
            ..civics(people, true, true)
        };
        assert_eq!(Era::of(chartered(119, true)), Era::Town);
        assert_eq!(Era::of(chartered(120, false)), Era::City);
        assert_eq!(Era::of(chartered(900, false)), Era::City);
        assert_eq!(Era::of(chartered(500, true)), Era::Metropolis);
        assert!(Era::Band < Era::Town && Era::City < Era::Metropolis);
        assert_eq!(Era::Town.term(), 4);
        assert!(Era::City.campaigns() && !Era::Village.campaigns());
        assert_eq!(Era::City.title(), "mayor");
    }

    #[test]
    fn every_office_is_held_best_by_someone_of_a_kind() {
        for (index, office) in Office::ALL.into_iter().enumerate() {
            assert!(!office.name().is_empty() && !office.telling().is_empty());
            assert!(
                Office::ALL[..index]
                    .iter()
                    .all(|other| other.suits() != office.suits())
            );
        }
    }

    #[test]
    fn appeals_take_time_and_rest() {
        for appeal in Appeal::ALL {
            assert!(appeal.seconds() >= 3_600 && appeal.rest_days() >= 1);
            assert!(!appeal.name().is_empty() && !appeal.effect().is_empty());
        }
        assert_eq!(Appeal::Speech.coins(), 0);
        assert!(Appeal::Broadcast.coins() > Appeal::Newspaper.coins());
    }
}
