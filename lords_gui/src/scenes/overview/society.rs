use egui::RichText;
use lords_sim::people::{APPRENTICE_AGE, Bond, MASTERY, Role};
use lords_sim::politics::Issue;
use lords_sim::view::{KinView, PersonView};
use lords_sim::{Command, Sex, View};

use crate::icons;

const ROW_SPACING: egui::Vec2 = egui::vec2(16.0, 6.0);
const SECTION_SPACING: f32 = 12.0;
const ROLE_WIDTH: f32 = 130.0;

/// How many people are listed at once.
const PAGE: usize = 40;

/// Shows the people the player character lives with and knows of, with the means to set
/// them to work and to seek them out. Returns what was asked for.
pub fn show(ui: &mut egui::Ui, view: &View) -> Option<Command> {
    let mut command = None;

    let leads = view
        .settlement
        .as_ref()
        .is_none_or(|town| town.politics.leads);
    let (with_you, others): (Vec<&PersonView>, Vec<&PersonView>) =
        view.people.iter().partition(|person| person.member);
    if !view.kin.is_empty() {
        ui.label(RichText::new("Your kin").strong());
        for one in view.kin.iter() {
            let words = kinsman(one);
            if one.alive {
                ui.label(words);
            } else {
                ui.label(RichText::new(words).weak());
            }
        }
        ui.add_space(SECTION_SPACING);
    }
    ui.horizontal(|ui| {
        let title = match &view.settlement {
            Some(town) => format!("The people of {}", town.name),
            None => "Your people".to_owned(),
        };
        ui.label(RichText::new(title).strong());
        ui.label(RichText::new(with_you.len().to_string()).weak());
    });
    if with_you.is_empty() {
        ui.label(RichText::new("You live alone.").weak());
    } else {
        // Those closest to the player character come first, a page at a time.
        let page = pager(ui, with_you.len());
        egui::Grid::new("your_people")
            .num_columns(4)
            .min_row_height(ui.spacing().interact_size.y)
            .spacing(ROW_SPACING)
            .show(ui, |ui| {
                for person in with_you.into_iter().skip(page * PAGE).take(PAGE) {
                    command = command.take().or(member_row(ui, person, leads));
                    ui.end_row();
                }
            });
    }
    ui.add_space(SECTION_SPACING);

    ui.horizontal(|ui| {
        ui.label(RichText::new("Others in the forest").strong());
        ui.label(RichText::new(others.len().to_string()).weak());
    });
    if others.is_empty() {
        ui.label(RichText::new("Nobody that you know of. Watch for smoke.").weak());
    } else {
        egui::Grid::new("other_people")
            .num_columns(3)
            .min_row_height(ui.spacing().interact_size.y)
            .spacing(ROW_SPACING)
            .show(ui, |ui| {
                for person in others {
                    ui.label(title(person));
                    ui.label(RichText::new(standing(person)).weak());
                    if ui.button("Walk to").clicked() {
                        command = Some(Command::WalkTo(person.subject));
                    }
                    ui.end_row();
                }
            });
    }
    ui.add_space(SECTION_SPACING);

    command
}

/// Shows the means to leaf through `people` rows a page at a time, if they take more
/// than one page, and returns the page that is open.
fn pager(ui: &mut egui::Ui, people: usize) -> usize {
    let pages = people.div_ceil(PAGE).max(1);
    let id = egui::Id::unique("people_page");
    let mut page: usize = ui.data(|data| data.get_temp(id)).unwrap_or(0);
    page = page.min(pages - 1);
    if pages == 1 {
        return page;
    }

    ui.horizontal(|ui| {
        if ui
            .add_enabled(page > 0, egui::Button::new("Earlier"))
            .clicked()
        {
            page -= 1;
        }
        ui.label(RichText::new(leaf(page, people)).weak());
        if ui
            .add_enabled(page + 1 < pages, egui::Button::new("Further"))
            .clicked()
        {
            page += 1;
        }
    });
    ui.data_mut(|data| data.insert_temp(id, page));
    page
}

/// Returns the words for one of the kin of the player character.
fn kinsman(kin: &KinView) -> String {
    let fate = if !kin.alive {
        format!("died at {}", kin.age)
    } else if kin.here {
        format!("{}", kin.age)
    } else {
        format!("{}, gone from here", kin.age)
    };
    format!("{}: {}, {fate}", kin.relation, kin.name)
}

/// Returns the words for which of `people` rows page number `page` shows.
fn leaf(page: usize, people: usize) -> String {
    let first = page * PAGE + 1;
    let last = ((page + 1) * PAGE).min(people);
    format!("{first} to {last} of {people}")
}

fn member_row(ui: &mut egui::Ui, person: &PersonView, leads: bool) -> Option<Command> {
    let mut command = None;

    ui.label(title(person)).on_hover_text(leanings(person));
    ui.label(RichText::new(person.bond.name()).weak());
    if person.works && !leads {
        ui.label(person.role.name())
            .on_hover_text("Only whoever leads sets people to work");
    } else if person.works {
        let mut role = person.role;
        egui::ComboBox::from_id_salt(("role", person.id))
            .width(ROLE_WIDTH)
            .selected_text(role.name())
            .show_ui(ui, |ui| {
                for choice in Role::ALL {
                    ui.selectable_value(&mut role, choice, choice.name())
                        .on_hover_text(choice.work());
                }
            })
            .response
            .on_hover_text(person.role.work());
        if role != person.role {
            command = Some(Command::Assign(person.id, role));
        }
    } else if person.bond == Bond::Child && person.age >= APPRENTICE_AGE {
        // A child of the player character can be set to learning a trade.
        let mut role = person.role;
        let learning = if person.apprenticed {
            format!("Learning: {}", role.name())
        } else {
            "Apprentice to…".to_owned()
        };
        egui::ComboBox::from_id_salt(("apprentice", person.id))
            .width(ROLE_WIDTH)
            .selected_text(learning)
            .show_ui(ui, |ui| {
                for choice in Role::ALL {
                    ui.selectable_value(&mut role, choice, choice.name())
                        .on_hover_text(choice.work());
                }
            })
            .response
            .on_hover_text(
                "A child of eight or more learns the trade you choose until they are old \
                 enough to work, and starts at it with what they have learned.",
            );
        if role != person.role {
            command = Some(Command::Apprentice(person.id, role));
        }
    } else {
        ui.label(RichText::new("Too young to work").weak());
    }
    ui.label(RichText::new(standing(person)).weak());

    command
}

/// Returns the words for who someone is: their name once they have been met, and their
/// age.
fn title(person: &PersonView) -> String {
    let icon = icons::sex(person.sex);
    if person.met {
        format!("{icon} {}, {}", person.name, person.age)
    } else {
        let what = match person.sex {
            Sex::Female => "A woman",
            Sex::Male => "A man",
        };
        format!("{icon} {what} you have not met")
    }
}

/// Returns the words for what someone holds, and how they are thought of.
fn leanings(person: &PersonView) -> String {
    let mut held: Vec<&str> = Issue::ALL
        .into_iter()
        .filter_map(|issue| {
            let (one, other) = issue.sides();
            match person.views[issue as usize] {
                ..0 => Some(one),
                0 => None,
                _ => Some(other),
            }
        })
        .collect();
    if held.is_empty() {
        held.push("No strong views");
    }

    let mut words = format!(
        "{}. Thinks {} of 100 of you, and is held in {} of 100 by the others.",
        held.join(", "),
        person.regard,
        person.esteem
    );
    for one in &person.traits {
        words.push_str(&format!(" {}: {}.", one.name(), one.telling()));
    }
    if person.works {
        words.push_str(&format!(" Skill at their trade: {} of 100.", person.skill));
    }
    words
}

/// Returns the words for how things stand with someone.
fn standing(person: &PersonView) -> String {
    let mut words = Vec::new();

    if person.bond != Bond::Partner
        && let Some(partner) = &person.partner
    {
        words.push(format!("with {partner}"));
    }
    if person.expecting {
        words.push("with child".to_owned());
    }
    if person.member && person.works && person.skill >= MASTERY {
        words.push("a master".to_owned());
    }
    if person.hired {
        words.push("thinking for you".to_owned());
    }
    if person.member && person.backs_you {
        words.push("backs you".to_owned());
    }
    if person.hungry {
        words.push("hungry".to_owned());
    }
    if !person.member && person.met {
        words.push(
            match person.regard {
                0..10 => "wary of you",
                10..30 => "warming to you",
                30..60 => "trusts you",
                _ => "fond of you",
            }
            .to_owned(),
        );
    }

    words.join(", ")
}

#[cfg(test)]
mod tests {
    use lords_sim::{Name, Simulation, Subject};

    use super::*;
    use crate::scenes::harness::Harness;

    fn person(met: bool, member: bool, regard: u8) -> PersonView {
        let mut world = lords_sim::World::default();
        let id = world.add_person(lords_sim::people::Person::new(
            Name::default(),
            Sex::Male,
            lords_sim::Timestamp::EPOCH,
            lords_sim::Location::default(),
        ));

        PersonView {
            id,
            subject: Subject::Person(id),
            name: "Eero Koski".to_owned(),
            sex: Sex::Male,
            age: 30,
            bond: Bond::Stranger,
            member,
            met,
            role: Role::Forager,
            works: true,
            regard,
            expecting: false,
            hungry: false,
            views: [-1, 0, 2, 0],
            esteem: 40,
            hired: false,
            backs_you: false,
            traits: vec![lords_sim::people::Trait::Bold],
            skill: 30,
            apprenticed: false,
            partner: None,
        }
    }

    #[test]
    fn someone_is_named_only_once_met() {
        assert!(title(&person(true, false, 0)).ends_with("Eero Koski, 30"));
        assert!(title(&person(false, false, 0)).ends_with("A man you have not met"));
    }

    #[test]
    fn leanings_tell_what_someone_holds_and_how_they_are_thought_of() {
        assert_eq!(
            leanings(&person(true, true, 50)),
            "All is shared, Everyone has a say. Thinks 50 of 100 of you, and is held in 40 \
             of 100 by the others. Bold: is worth two as a guard. Skill at their trade: 30 \
             of 100."
        );
    }

    #[test]
    fn standing_tells_how_things_are_with_someone() {
        assert_eq!(standing(&person(true, false, 5)), "wary of you");
        assert_eq!(standing(&person(true, false, 45)), "trusts you");
        assert_eq!(standing(&person(false, false, 0)), "");

        let mut villager = person(true, true, 50);
        villager.partner = Some("Aino".to_owned());
        villager.hungry = true;
        assert_eq!(standing(&villager), "with Aino, hungry");
        villager.skill = 80;
        assert_eq!(standing(&villager), "with Aino, a master, hungry");
    }

    #[test]
    fn kin_are_told_by_what_they_are_and_what_became_of_them() {
        let mut kin = KinView {
            name: "Aino Koski".to_owned(),
            relation: "Child",
            age: 8,
            alive: true,
            here: true,
        };
        assert_eq!(kinsman(&kin), "Child: Aino Koski, 8");
        kin.here = false;
        assert_eq!(kinsman(&kin), "Child: Aino Koski, 8, gone from here");
        kin.alive = false;
        assert_eq!(kinsman(&kin), "Child: Aino Koski, died at 8");
    }

    #[test]
    fn pages_are_told_by_the_rows_they_show() {
        assert_eq!(leaf(0, 95), "1 to 40 of 95");
        assert_eq!(leaf(2, 95), "81 to 95 of 95");
    }

    #[test]
    fn tab_draws_alone_and_for_a_settlement_of_many() {
        let harness = Harness::new();
        let alone = Simulation::generate(Name::default(), Sex::Female, 1).view();
        harness.show(|ui| assert_eq!(show(ui, &alone), None));

        let mut town = alone.clone();
        let mut people = vec![person(false, false, 0)];
        people.extend((0..100).map(|_| person(true, true, 50)));
        town.people = people.into();
        town.settlement = Some(std::sync::Arc::new(
            crate::scenes::overview::town::tests::settlement(true),
        ));
        harness.show(|ui| assert_eq!(show(ui, &town), None));
    }
}
