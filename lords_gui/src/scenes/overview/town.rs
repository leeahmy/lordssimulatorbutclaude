use egui::RichText;
use lords_sim::item::Inventory;
use lords_sim::settlement::{BUILDINGS, Building, HIGHEST_TAX, Ledger};
use lords_sim::view::SettlementView;
use lords_sim::world::Kind;
use lords_sim::{Command, Item, View};

use crate::scenes::count;

const SECTION_SPACING: f32 = 12.0;
const BAR_WIDTH: f32 = 140.0;
const TAX_WIDTH: f32 = 160.0;
const ITEM_WIDTH: f32 = 140.0;

/// The least a share in a workplace costs, and the most that is given away at once.
const LEAST_SHARE: i64 = 200;
const LARGEST_GIFT: u32 = 100_000;

/// The most of anything that can be sent for at once.
const LARGEST_ORDER: u32 = 500;

/// Shows the settlement: how its people fare, what it has in store, what stands built
/// and can be, who works at what, and what it has in its treasury. Returns what was
/// asked for.
pub fn show(ui: &mut egui::Ui, view: &View) -> Option<Command> {
    let Some(town) = &view.settlement else {
        ui.label(RichText::new("No settlement").strong());
        ui.label(
            RichText::new(
                "Raise a cairn of ten stones, from the Crafting tab, where you mean to \
                 stay. Those who join you will live and work there, and what you build \
                 together can grow from a few huts into a city.",
            )
            .weak(),
        );
        return None;
    };
    let leads = town.politics.leads;
    let mut command = None;

    ui.label(RichText::new(&town.name).heading().strong());
    ui.label(
        RichText::new(format!(
            "A {} in the {}",
            town.politics.era.name().to_lowercase(),
            town.age.name()
        ))
        .weak(),
    );
    ui.add_space(4.0);
    summary(ui, town);
    ui.add_space(SECTION_SPACING);

    ui.label(RichText::new("Store").strong());
    ui.label(RichText::new(listing(&town.store)).weak())
        .on_hover_text(
            "Stand by the cairn, a storehouse, or the market to take from it and put into it",
        );
    ui.add_space(SECTION_SPACING);

    command = command.or(buildings(ui, town, leads));
    ui.add_space(SECTION_SPACING);

    work(ui, town);
    ui.add_space(SECTION_SPACING);

    if town.coined {
        command = command.or(treasury(ui, town, leads));
        ui.add_space(SECTION_SPACING);
        command = command.or(fortune(ui, view, town));
        ui.add_space(SECTION_SPACING);
    }

    command
}

fn summary(ui: &mut egui::Ui, town: &SettlementView) {
    ui.horizontal_wrapped(|ui| {
        let dot = |ui: &mut egui::Ui| ui.label(RichText::new("·").weak());
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
        dot(ui);
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
            dot(ui);
            ui.label(format!("{} measures of rye growing", town.growing));
        }
        dot(ui);
        let mood = RichText::new(mood(town.contentment));
        let mood = if town.contentment < 40 {
            mood.color(ui.visuals().warn_fg_color)
        } else {
            mood
        };
        ui.label(mood).on_hover_text(format!(
            "{} of 100. People are content when they are fed, housed, healthy, and lightly \
             taxed, and have work, ale, and a school. Content people draw others to settle. \
             Below 25 they start to leave.",
            town.contentment
        ));
        if town.literacy > 0 {
            dot(ui);
            ui.label(format!("{} in 100 can read", town.literacy))
                .on_hover_text(
                    "Reading spreads where there are teachers. The more who read, the further \
                     thinkers get, and the more a notice or a newspaper reaches.",
                );
        }
        if town.tools > 100 {
            dot(ui);
            ui.label(format!("tools add {}% to all work", town.tools - 100))
                .on_hover_text("For as long as there are tools in the store. They wear out.");
        }
        if town.crime >= 15 {
            dot(ui);
            let words = RichText::new(lawlessness(town.crime));
            let words = if town.crime >= 40 {
                words.color(ui.visuals().warn_fg_color)
            } else {
                words
            };
            ui.label(words).on_hover_text(format!(
                "{} of 100. Crime grows where people are idle, crowded, and discontented,                  and with the size of the place. Guards, a courthouse with clerks, and                  priests keep it down. Thieves take from the store and the treasury, and                  nobody is content where it is rife.",
                town.crime
            ));
        }
        if town.pollution >= 10 {
            dot(ui);
            let words = if town.pollution >= 30 {
                RichText::new("the air is foul").color(ui.visuals().warn_fg_color)
            } else {
                RichText::new("there is smoke in the air")
            };
            ui.label(words).on_hover_text(format!(
                "{} of 100. Kilns, brickworks, steelworks, factories, mills, and above all \
                 power stations that burn coal foul the air and the water. More sicken, \
                 and people are less content. Waterworks, a park, and clean power help.",
                town.pollution
            ));
        }
        if town.defence > 0 {
            dot(ui);
            ui.label(format!("defence worth {}", town.defence))
                .on_hover_text("See the World tab for who it has to hold against");
        }
        if town.running.power {
            dot(ui);
            ui.label("the power is on");
        }
    });
}

/// Returns the words for how much crime there is.
fn lawlessness(crime: u8) -> &'static str {
    match crime {
        0..15 => "the peace is kept",
        15..40 => "there is some thieving",
        40..70 => "thieves are bold",
        _ => "nobody is safe",
    }
}

/// Returns the words for how content people are.
fn mood(contentment: u8) -> &'static str {
    match contentment {
        0..25 => "people are leaving",
        25..40 => "people are restless",
        40..60 => "people get by",
        60..80 => "people are content",
        _ => "people are thriving",
    }
}

fn buildings(ui: &mut egui::Ui, town: &SettlementView, leads: bool) -> Option<Command> {
    let mut command = None;

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

    let known = |plan: &Building| {
        plan.needs
            .is_none_or(|needed| town.advances.contains(&needed))
    };
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new("Plan:").weak());
        for plan in BUILDINGS.iter().filter(|plan| known(plan)) {
            let stocked = plan
                .materials
                .iter()
                .all(|(item, count)| town.store.count(*item) >= *count);
            let paid = !town.coined || i64::from(plan.coins) <= town.treasury;
            let name = Kind::Structure(plan.kind).name();

            let button = ui
                .add_enabled(stocked && paid && leads, egui::Button::new(name))
                .on_hover_text(blueprint(plan, town.coined))
                .on_disabled_hover_text(blueprint(plan, town.coined));
            if button.clicked() {
                command = Some(Command::Plan(plan.kind));
            }
        }
    });
    let beyond = BUILDINGS.iter().filter(|plan| !known(plan)).count();
    if beyond > 0 {
        ui.label(
            RichText::new(format!(
                "{beyond} more kinds of building can be planned once your people have \
                 worked out how."
            ))
            .weak(),
        );
    }
    if !leads {
        ui.label(RichText::new("Only whoever leads plans buildings.").weak());
    }

    command
}

fn work(ui: &mut egui::Ui, town: &SettlementView) {
    ui.label(RichText::new("Work").strong()).on_hover_text(
        "How many work at each trade. Trades that take a building have only as many places \
         as their buildings give.",
    );
    if town.jobs.is_empty() {
        ui.label(RichText::new("Nobody works here yet.").weak());
        return;
    }

    ui.horizontal_wrapped(|ui| {
        for (role, working, places) in &town.jobs {
            let words = match places {
                Some(places) => format!("{} {working}/{places}", role.name()),
                None => format!("{} {working}", role.name()),
            };
            let unfilled = places.is_some_and(|places| *working < places);
            let words = if unfilled {
                RichText::new(words).weak()
            } else {
                RichText::new(words)
            };
            ui.label(words).on_hover_text(role.work());
            ui.label(RichText::new("·").weak());
        }
    });
}

fn treasury(ui: &mut egui::Ui, town: &SettlementView, leads: bool) -> Option<Command> {
    let mut command = None;

    ui.label(RichText::new("Treasury").strong());
    ui.label(format!(
        "{} in the treasury",
        count(town.treasury.max(0) as u64, "coin")
    ));
    ui.label(RichText::new(accounts(&town.ledger)).weak())
        .on_hover_text(
            "Taxes are raised on what people earn and on what they sell outside. Those who \
             build, think, teach, heal, and keep the rolls are paid out of the treasury, as \
             is the upkeep of everything but houses. When it runs dry they do half of what \
             they would, and people are not content.",
        );
    if town.ledger.unpaid {
        ui.label(
            RichText::new("There was not enough to pay everyone yesterday.")
                .color(ui.visuals().warn_fg_color),
        );
    }

    ui.horizontal(|ui| {
        ui.label("Taxes");
        let mut tax = town.tax;
        ui.spacing_mut().slider_width = TAX_WIDTH;
        let slider = egui::Slider::new(&mut tax, 0..=HIGHEST_TAX).suffix(" in 100");
        let moved = ui
            .add_enabled(leads, slider)
            .on_hover_text("Every point over ten weighs on how content people are")
            .on_disabled_hover_text("Only whoever leads sets the taxes")
            .changed();
        if moved {
            command = Some(Command::SetTax(tax));
        }
    });

    if !town.monetized {
        ui.label(RichText::new("Without a market, nothing is bought or sold.").weak());
        return command;
    }
    ui.label(
        RichText::new(format!(
            "The merchants can trade {} worth a day.",
            count(u64::from(town.running.trade), "coin")
        ))
        .weak(),
    )
    .on_hover_text(
        "What the store holds too much of is sold outside, and what is sent for is bought \
         there for three tenths over what it is worth. More merchants, sailors, and \
         transport workers trade more.",
    );
    for (item, wanted) in &town.orders {
        ui.horizontal(|ui| {
            ui.label(format!(
                "{wanted} × {} sent for",
                item.name().to_lowercase()
            ));
            if ui
                .add_enabled(leads, egui::Button::new("Call off"))
                .clicked()
            {
                command = Some(Command::Order(*item, 0));
            }
        });
    }
    if leads {
        command = command.or(send_for(ui));
    }

    command
}

/// Shows what the player character has of their own, and the means to put it to use:
/// to buy into workplaces, to give to the treasury, and to give to those who think.
fn fortune(ui: &mut egui::Ui, view: &View, town: &SettlementView) -> Option<Command> {
    let mut command = None;

    ui.label(RichText::new("Your fortune").strong());
    ui.label(wealth(view.purse, view.income)).on_hover_text(
        "A share in a workplace pays you a quarter of what those who work there earn, \
         every day, less the taxes on it. There is one share to be had in each that stands.",
    );
    if !view.holdings.is_empty() {
        let held: Vec<String> = view
            .holdings
            .iter()
            .map(|(kind, shares)| {
                let name = Kind::Structure(*kind).name().to_lowercase();
                format!("{shares} × {name}")
            })
            .collect();
        ui.label(RichText::new(format!("You own shares in: {}.", held.join(", "))).weak());
    }
    if !town.monetized {
        return None;
    }

    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new("Buy into:").weak());
        for (kind, standing) in &town.buildings {
            let plan = BUILDINGS.iter().find(|plan| plan.kind == *kind);
            let Some(plan) = plan.filter(|plan| plan.jobs.is_some()) else {
                continue;
            };
            let held = view.holdings.iter().find(|(held, _)| held == kind);
            if held.is_some_and(|(_, shares)| shares >= standing) {
                continue;
            }
            let price = i64::from(plan.coins).max(LEAST_SHARE);
            let name = Kind::Structure(*kind).name();
            let button = ui
                .add_enabled(view.purse >= price, egui::Button::new(name))
                .on_hover_text(format!("{price} coins"))
                .on_disabled_hover_text(format!("{price} coins"));
            if button.clicked() {
                command = Some(Command::Invest(*kind));
            }
        }
    });

    let id = egui::Id::unique("gift");
    let mut gift: u32 = ui.data(|data| data.get_temp(id)).unwrap_or(50);
    ui.horizontal(|ui| {
        ui.label("Give");
        ui.add(egui::DragValue::new(&mut gift).range(1..=LARGEST_GIFT));
        let can = view.purse >= i64::from(gift);
        let to_all = ui
            .add_enabled(can, egui::Button::new("to the treasury"))
            .on_hover_text("People think a point better of you for every 50 coins, up to 5");
        if to_all.clicked() {
            command = Some(Command::Donate(gift));
        }
        let to_minds = ui
            .add_enabled(
                can && town.pursuit.is_some(),
                egui::Button::new("to those who think"),
            )
            .on_hover_text("Every 5 coins bring what is being worked out a thinker's day on");
        if to_minds.clicked() {
            command = Some(Command::Endow(gift));
        }
    });
    ui.data_mut(|data| data.insert_temp(id, gift));

    command
}

/// Returns the words for what the player character has and what it brings in.
fn wealth(purse: i64, income: i64) -> String {
    let purse = match purse {
        0.. => format!("You have {} in your purse", count(purse as u64, "coin")),
        _ => format!("You owe the market {}", count(purse.unsigned_abs(), "coin")),
    };
    if income > 0 {
        format!("{purse}, and your shares paid {income} yesterday.")
    } else {
        format!("{purse}.")
    }
}

/// Shows the means to have the merchants buy something from outside.
fn send_for(ui: &mut egui::Ui) -> Option<Command> {
    let id = egui::Id::unique("send_for");
    let (mut item, mut wanted): (Item, u32) = ui
        .data(|data| data.get_temp(id))
        .unwrap_or((Item::Tools, 10));
    let mut command = None;

    ui.horizontal(|ui| {
        ui.label("Send for");
        ui.add(egui::DragValue::new(&mut wanted).range(1..=LARGEST_ORDER));
        egui::ComboBox::from_id_salt("send_for_item")
            .width(ITEM_WIDTH)
            .selected_text(item.name())
            .show_ui(ui, |ui| {
                for choice in Item::ALL {
                    ui.selectable_value(&mut item, choice, choice.name());
                }
            });
        if ui.button("Order").clicked() {
            command = Some(Command::Order(item, wanted));
        }
    });
    ui.data_mut(|data| data.insert_temp(id, (item, wanted)));

    command
}

/// Returns the words for what came into the treasury and went out of it in a day.
fn accounts(ledger: &Ledger) -> String {
    let mut words = vec![format!("Yesterday {} came in in taxes", ledger.taxes)];
    if ledger.interest > 0 {
        words.push(format!("{} in interest", ledger.interest));
    }
    words.push(format!("{} went on pay", ledger.wages));
    words.push(format!("{} on upkeep", ledger.upkeep));
    if ledger.purchases > 0 {
        words.push(format!("{} on what was sent for", ledger.purchases));
    }
    format!(
        "{}. Goods worth {} were sold outside.",
        words.join(", "),
        ledger.sales
    )
}

/// Returns the words for what it takes to put up a building and what it is good for.
fn blueprint(plan: &Building, coined: bool) -> String {
    let materials: Vec<String> = plan
        .materials
        .iter()
        .map(|(item, count)| format!("{count} × {}", item.name().to_lowercase()))
        .collect();
    let needs = plan
        .needs
        .map(|needed| format!(" Takes knowing {}.", needed.name().to_lowercase()))
        .unwrap_or_default();
    let coins = if coined && plan.coins > 0 {
        format!(", {} coins from the treasury,", plan.coins)
    } else {
        String::new()
    };

    format!(
        "{}. Takes {} from the store{coins} and {} hours of work.{needs}",
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
pub(super) mod tests {
    use lords_sim::people::Role;
    use lords_sim::settlement::{Advance, Age, Running};
    use lords_sim::world::StructureKind;
    use lords_sim::{Name, Sex, Simulation};

    use super::*;
    use crate::scenes::harness::Harness;

    /// A town in the Iron Age with coin and a market.
    pub(in crate::scenes::overview) fn settlement(leads: bool) -> SettlementView {
        let mut store = Inventory::default();
        store.add(Item::Log, 50);
        store.add(Item::Stone, 40);

        SettlementView {
            name: "Lordstead".to_owned(),
            founded: lords_sim::Timestamp::EPOCH,
            people: 31,
            housing: 28,
            food_days: 1,
            store,
            advances: vec![
                Advance::Pottery,
                Advance::Masonry,
                Advance::Mining,
                Advance::Charcoal,
                Advance::Bronze,
                Advance::Writing,
                Advance::Arithmetic,
                Advance::Coinage,
            ],
            pursuit: Some(Advance::Farming),
            insight: 12,
            growing: 90,
            buildings: vec![(StructureKind::Hut, 2), (StructureKind::Market, 1)],
            politics: crate::scenes::overview::politics::tests::politics(leads),
            sites: vec![lords_sim::view::SiteView {
                kind: StructureKind::Storehouse,
                work: 10,
                hours: 64,
            }],
            age: Age::Iron,
            coined: true,
            monetized: true,
            treasury: 420,
            tax: 12,
            ledger: Ledger {
                taxes: 14,
                sales: 60,
                interest: 0,
                wages: 9,
                upkeep: 2,
                purchases: 26,
                unpaid: true,
            },
            orders: vec![(Item::Tools, 4)],
            literacy: 7,
            contentment: 38,
            running: Running {
                trade: 90,
                ..Running::default()
            },
            tools: 115,
            defence: 9,
            crime: 45,
            pollution: 34,
            chronicle: vec![
                lords_sim::settlement::Chronicled {
                    time: lords_sim::Timestamp::EPOCH,
                    text: "Ada Lord raised a cairn and founded the place.".to_owned(),
                },
                lords_sim::settlement::Chronicled {
                    time: lords_sim::Timestamp::EPOCH,
                    text: "The first hut was built.".to_owned(),
                },
            ],
            jobs: vec![
                (Role::Fisher, 6, None),
                (Role::Merchant, 2, Some(3)),
                (Role::Smith, 2, Some(2)),
            ],
        }
    }

    #[test]
    fn wealth_is_told_with_what_it_brings_in() {
        assert_eq!(wealth(0, 0), "You have 0 coins in your purse.");
        assert_eq!(
            wealth(240, 6),
            "You have 240 coins in your purse, and your shares paid 6 yesterday."
        );
        assert_eq!(wealth(-30, 0), "You owe the market 30 coins.");
    }

    #[test]
    fn mood_is_told_in_words() {
        assert_eq!(mood(10), "people are leaving");
        assert_eq!(mood(30), "people are restless");
        assert_eq!(mood(50), "people get by");
        assert_eq!(mood(70), "people are content");
        assert_eq!(mood(95), "people are thriving");
        assert_eq!(lawlessness(5), "the peace is kept");
        assert_eq!(lawlessness(45), "thieves are bold");
        assert_eq!(lawlessness(90), "nobody is safe");
    }

    #[test]
    fn blueprint_and_store_are_listed_in_words() {
        assert_eq!(
            blueprint(&BUILDINGS[0], true),
            "Houses four. Takes 12 × log, 20 × spruce bough from the store and 48 hours of work."
        );
        assert!(blueprint(&BUILDINGS[10], false).ends_with("Takes knowing masonry."));
        let market = BUILDINGS
            .iter()
            .find(|plan| plan.kind == StructureKind::Market)
            .expect("there is a market");
        assert!(blueprint(market, true).contains("300 coins from the treasury"));
        assert!(!blueprint(market, false).contains("coins"));

        let mut store = Inventory::default();
        assert_eq!(listing(&store), "Empty");
        store.add(Item::Log, 4);
        store.add(Item::Grain, 20);
        assert_eq!(listing(&store), "4 × log, 20 × rye");
    }

    #[test]
    fn accounts_tell_what_came_in_and_what_went_out() {
        assert_eq!(
            accounts(&settlement(true).ledger),
            "Yesterday 14 came in in taxes, 9 went on pay, 2 on upkeep, 26 on what was \
             sent for. Goods worth 60 were sold outside."
        );
    }

    #[test]
    fn tab_draws_alone_for_a_newcomer_and_for_a_leader_with_a_treasury() {
        let harness = Harness::new();
        let alone = Simulation::new(Name::default(), Sex::Female).view();
        harness.show(|ui| assert_eq!(show(ui, &alone), None));

        let newcomer = Simulation::generate(Name::default(), Sex::Female, 1).view();
        harness.show(|ui| assert_eq!(show(ui, &newcomer), None));

        for leads in [false, true] {
            let mut town = newcomer.clone();
            town.settlement = Some(std::sync::Arc::new(settlement(leads)));
            harness.show(|ui| assert_eq!(show(ui, &town), None));
        }
    }
}
