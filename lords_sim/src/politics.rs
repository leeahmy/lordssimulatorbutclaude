//! What the people of a settlement hold on how to live together, who they follow, and the
//! laws they live by.

use serde::{Deserialize, Serialize};

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
}

impl Law {
    pub const ALL: [Self; 8] = [
        Self::SharedTable,
        Self::OwnHearths,
        Self::OldCustoms,
        Self::Schooling,
        Self::EldersWord,
        Self::OpenMoot,
        Self::ClosedGates,
        Self::OpenGates,
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
        }
    }

    /// Returns what living by the law comes to.
    pub fn effect(self) -> &'static str {
        match self {
            Self::SharedTable => {
                "Everyone eats at one table: a tenth less food is needed, and a little less \
                 is brought in"
            }
            Self::OwnHearths => {
                "Each household keeps part of what it brings in: more food is brought in, \
                 and more is eaten"
            }
            Self::OldCustoms => {
                "People bear hunger longer before they leave, and thinkers get less far"
            }
            Self::Schooling => {
                "Children learn until fourteen before they work, and thinkers get half \
                 again as far"
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
        }
    }

    pub const fn issue(self) -> Issue {
        match self {
            Self::SharedTable | Self::OwnHearths => Issue::Property,
            Self::OldCustoms | Self::Schooling => Issue::Change,
            Self::EldersWord | Self::OpenMoot => Issue::Rule,
            Self::ClosedGates | Self::OpenGates => Issue::Strangers,
        }
    }

    /// Returns which side of its issue the law is on: -1 or 1.
    pub const fn side(self) -> i8 {
        match self {
            Self::SharedTable | Self::OldCustoms | Self::EldersWord | Self::ClosedGates => -1,
            Self::OwnHearths | Self::Schooling | Self::OpenMoot | Self::OpenGates => 1,
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
        }
    }
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
}

impl Era {
    /// The fewest people a village and a town have.
    pub const VILLAGE: u32 = 8;
    pub const TOWN: u32 = 25;

    /// Returns the era of a settlement of `people`, with or without `writing` and a
    /// meeting `hall`.
    pub const fn of(people: u32, writing: bool, hall: bool) -> Self {
        if people >= Self::TOWN && writing && hall {
            Self::Town
        } else if people >= Self::VILLAGE {
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
                 are put to all the grown-ups."
            }
        }
    }

    /// Returns how many years pass between the choosing of leaders.
    pub const fn term(self) -> u64 {
        match self {
            Self::Band | Self::Village => 1,
            Self::Town => 4,
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
}

impl Appeal {
    pub const ALL: [Self; 3] = [Self::Speech, Self::Feast, Self::Notices];

    pub fn name(self) -> &'static str {
        match self {
            Self::Speech => "Speak to the village",
            Self::Feast => "Hold a feast",
            Self::Notices => "Put up notices",
        }
    }

    pub fn effect(self) -> &'static str {
        match self {
            Self::Speech => {
                "An hour by the cairn, once a day. Wins over those who think like you, and \
                 puts off those who think the opposite."
            }
            Self::Feast => {
                "Four hours, once a month, out of food you carry or have piled up by the \
                 cairn: 1,500 kcal for each of your people. Everyone thinks better of you."
            }
            Self::Notices => {
                "Two hours and five pieces of birch bark, once a week, once writing is \
                 known. Everyone grown-up thinks a little better of you."
            }
        }
    }

    /// Returns how many in-game seconds it takes.
    pub const fn seconds(self) -> u32 {
        match self {
            Self::Speech => 3_600,
            Self::Feast => 4 * 3_600,
            Self::Notices => 2 * 3_600,
        }
    }

    /// Returns how many days pass before it can be done again.
    pub const fn rest_days(self) -> u64 {
        match self {
            Self::Speech => 1,
            Self::Feast => 30,
            Self::Notices => 7,
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
                2
            );
            assert_eq!(Issue::ALL[issue as usize], issue);
        }
    }

    #[test]
    fn settlement_matures_with_people_a_hall_and_writing() {
        assert_eq!(Era::of(7, true, true), Era::Band);
        assert_eq!(Era::of(8, false, false), Era::Village);
        assert_eq!(Era::of(60, true, false), Era::Village);
        assert_eq!(Era::of(60, false, true), Era::Village);
        assert_eq!(Era::of(25, true, true), Era::Town);
        assert!(Era::Band < Era::Town);
        assert_eq!(Era::Town.term(), 4);
    }

    #[test]
    fn appeals_take_time_and_rest() {
        for appeal in Appeal::ALL {
            assert!(appeal.seconds() >= 3_600 && appeal.rest_days() >= 1);
            assert!(!appeal.name().is_empty() && !appeal.effect().is_empty());
        }
    }
}
