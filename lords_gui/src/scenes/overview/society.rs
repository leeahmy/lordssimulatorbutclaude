use egui::RichText;
use lords_sim::item::Inventory;
use lords_sim::people::{Bond, Role};
use lords_sim::politics::Issue;
use lords_sim::settlement::{Advance, BUILDINGS, Building};
use lords_sim::view::{PersonView, SettlementView};
use lords_sim::world::Kind;
use lords_sim::{Command, Sex, View};

use crate::icons;
use crate::scenes::count;

const ROW_SPACING: egui::Vec2 = egui::vec2(16.0, 6.0);
const SECTION_SPACING: f32 = 12.0;
const BAR_WIDTH: f32 = 140.0;
const ROLE_WIDTH: f32 = 110.0;

/// Shows the settlement and the people the player character knows of, with the means to
/// run the one and to seek out the others. Returns what was asked for.
pub fn show(ui: &mut egui::Ui, view: &View) -> Option<Command> {
    let mut command = None;

    match &view.settlement {
        Some(town) => command = settlement(ui, town),
        None => {
            ui.label(RichText::new("No settlement").strong());
            ui.label(
                RichText::new(
                    "Raise a cairn of ten stones, from the Crafting tab, where you mean to \
                     stay. Those who join you will live and work there.",
                )
                .weak(),
            );
        }
    }
    ui.add_space(SECTION_SPACING);

    let leads = view
        .settlement
        .as_ref()
        .is_none_or(|town| town.politics.leads);
    let (with_you, others): (Vec<&PersonView>, Vec<&PersonView>) =
        view.people.iter().partition(|person| person.member);
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
        egui::Grid::new("your_people")
            .num_columns(4)
            .min_row_height(ui.spacing().interact_size.y)
            .spacing(ROW_SPACING)
            .show(ui, |ui| {
                for person in with_you {
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

fn settlement(ui: &mut egui::Ui, town: &SettlementView) -> Option<Command> {
    let mut command = None;
    let leads = town.politics.leads;

    ui.label(RichText::new(&town.name).heading().strong());
    ui.horizontal_wrapped(|ui| {
        let crowded = town.people > town.housing;
        let housed = RichText::new(format!(
            "{} living here, houses for {}",
            town.people, town.housing
        ));
        if crowded {
            ui.label(housed.color(ui.visuals().warn_fg_color))
                .on_hover_text("Those without a house are at the mercy of the frost");
        } else {
            ui.label(housed);
        }
        ui.label(RichText::new("·").weak());
        let fed = RichText::new(format!(
            "food for {}",
            count(u64::from(town.food_days), "day")
        ));
        if town.food_days < 3 {
            ui.label(fed.color(ui.visuals().warn_fg_color));
        } else {
            ui.label(fed);
        }
        if town.growing > 0 {
            ui.label(RichText::new("·").weak());
            ui.label(format!("{} measures of rye growing", town.growing));
        }
    });
    ui.add_space(SECTION_SPACING);

    ui.label(RichText::new("Store").strong());
    ui.label(RichText::new(listing(&town.store)).weak())
        .on_hover_text("Stand by the cairn or a storehouse to take from it and put into it");
    ui.add_space(SECTION_SPACING);

    ui.label(RichText::new("Buildings").strong());
    if town.buildings.is_empty() && town.sites.is_empty() {
        ui.label(RichText::new("Nothing stands yet.").weak());
    }
    if !town.buildings.is_empty() {
        let built: Vec<String> = town
            .buildings
            .iter()
            .map(|(kind, count)| {
                let name = Kind::Structure(*kind).name();
                if *count > 1 {
                    format!("{name} ×{count}")
                } else {
                    name.to_owned()
                }
            })
            .collect();
        ui.label(built.join(", "));
    }
    for site in &town.sites {
        ui.horizontal(|ui| {
            ui.label(format!("{} being built", Kind::Structure(site.kind).name()));
            ui.add(
                egui::ProgressBar::new(site.work as f32 / site.hours.max(1) as f32)
                    .desired_width(BAR_WIDTH)
                    .text(format!("{} of {} h", site.work, site.hours)),
            );
        });
    }
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new("Plan:").weak());
        for plan in &BUILDINGS {
            let known = plan
                .needs
                .is_none_or(|needed| town.advances.contains(&needed));
            let stocked = plan
                .materials
                .iter()
                .all(|(item, count)| town.store.count(*item) >= *count);
            let name = Kind::Structure(plan.kind).name();

            let button = ui
                .add_enabled(known && stocked && leads, egui::Button::new(name))
                .on_hover_text(blueprint(plan))
                .on_disabled_hover_text(blueprint(plan));
            if button.clicked() {
                command = Some(Command::Plan(plan.kind));
            }
        }
    });
    ui.add_space(SECTION_SPACING);

    ui.label(RichText::new("Advances").strong());
    if !town.advances.is_empty() {
        let known: Vec<&str> = town.advances.iter().map(|advance| advance.name()).collect();
        ui.label(format!("Known: {}", known.join(", ")));
    }
    if let Some(pursuit) = town.pursuit {
        ui.horizontal(|ui| {
            ui.label(format!("Working out {}", pursuit.name().to_lowercase()))
                .on_hover_text(pursuit.use_of());
            ui.add(
                egui::ProgressBar::new(town.insight as f32 / pursuit.cost() as f32)
                    .desired_width(BAR_WIDTH)
                    .text(format!("{} of {}", town.insight, pursuit.cost())),
            );
        });
    }
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new("Pursue:").weak());
        let mut any = false;
        for advance in Advance::ALL {
            if town.advances.contains(&advance) {
                continue;
            }
            any = true;
            let within_reach = advance
                .requires()
                .is_none_or(|needed| town.advances.contains(&needed));
            let hint = match advance.requires().filter(|_| !within_reach) {
                Some(needed) => format!("{}. Takes {} first.", advance.use_of(), needed.name()),
                None => format!(
                    "{}. Takes {} of a thinker's work.",
                    advance.use_of(),
                    count(u64::from(advance.cost()), "day")
                ),
            };

            let button = egui::Button::new(advance.name()).selected(town.pursuit == Some(advance));
            let button = ui
                .add_enabled(within_reach && (leads || town.pursuit.is_none()), button)
                .on_hover_text(&hint)
                .on_disabled_hover_text(&hint);
            if button.clicked() {
                command = Some(Command::Pursue(advance));
            }
        }
        if !any {
            ui.label(RichText::new("There is nothing left to work out.").weak());
        }
    });

    command
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

    format!(
        "{}. Thinks {} of 100 of you, and is held in {} of 100 by the others.",
        held.join(", "),
        person.regard,
        person.esteem
    )
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

/// Returns the words for what it takes to put up a building and what it is good for.
fn blueprint(plan: &Building) -> String {
    let materials: Vec<String> = plan
        .materials
        .iter()
        .map(|(item, count)| format!("{count} × {}", item.name().to_lowercase()))
        .collect();
    let needs = plan
        .needs
        .map(|needed| format!(" Takes knowing {}.", needed.name().to_lowercase()))
        .unwrap_or_default();

    format!(
        "{}. Takes {} from the store and {} hours of work.{needs}",
        plan.use_of,
        materials.join(", "),
        plan.hours
    )
}

/// Returns the words for everything in `inventory`.
fn listing(inventory: &Inventory) -> String {
    if inventory.is_empty() {
        return "Empty".to_owned();
    }

    let stacks: Vec<String> = inventory
        .stacks()
        .iter()
        .map(|stack| format!("{} × {}", stack.count, stack.item.name().to_lowercase()))
        .collect();
    stacks.join(", ")
}

#[cfg(test)]
mod tests {
    use lords_sim::{Item, Name, Simulation, Subject};

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
             of 100 by the others."
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
    }

    #[test]
    fn blueprint_and_store_are_listed_in_words() {
        assert_eq!(
            blueprint(&BUILDINGS[0]),
            "Houses four. Takes 12 × log, 20 × spruce bough from the store and 48 hours of work."
        );
        assert!(blueprint(&BUILDINGS[10]).ends_with("Takes knowing masonry."));

        let mut store = Inventory::default();
        assert_eq!(listing(&store), "Empty");
        store.add(Item::Log, 4);
        store.add(Item::Grain, 20);
        assert_eq!(listing(&store), "4 × log, 20 × rye");
    }

    #[test]
    fn tab_draws_alone_and_for_a_settlement() {
        let harness = Harness::new();
        let alone = Simulation::generate(Name::default(), Sex::Female, 1).view();
        harness.show(|ui| assert_eq!(show(ui, &alone), None));

        let mut town = alone.clone();
        let mut store = Inventory::default();
        store.add(Item::Log, 50);
        town.people = vec![person(true, true, 50), person(false, false, 0)].into();
        town.settlement = Some(std::sync::Arc::new(SettlementView {
            name: "Lordstead".to_owned(),
            founded: town.time,
            people: 2,
            housing: 0,
            food_days: 1,
            store,
            advances: vec![Advance::Pottery],
            pursuit: Some(Advance::Farming),
            insight: 12,
            growing: 90,
            buildings: vec![(lords_sim::world::StructureKind::Hut, 2)],
            politics: crate::scenes::overview::politics::tests::politics(true),
            sites: vec![lords_sim::view::SiteView {
                kind: lords_sim::world::StructureKind::Storehouse,
                work: 10,
                hours: 64,
            }],
        }));
        harness.show(|ui| assert_eq!(show(ui, &town), None));
    }
}
