//! The guide to the game, shown from the Help menu.

use egui::RichText;

const DIALOG_WIDTH: f32 = 520.0;

/// How much of the window's height the guide takes at most.
const MAX_HEIGHT_SHARE: f32 = 0.7;

const CHAPTERS: [(&str, &[&str]); 20] = [
    (
        "The life",
        &[
            "You are alone in a northern forest by a lake, with a knife, a fire steel, and a \
             little bread. It is the first of May. Nothing is asked of you but to live, and \
             winter is the hardest part of that.",
            "The bars on the left tell how your body is doing. Hunger kills in weeks, thirst \
             in days, and the cold in hours.",
        ],
    ),
    (
        "Getting around",
        &[
            "Click something on the map, or its Walk to button in the list, to walk to it. \
             You can only walk to what you have seen. Scroll over the map to zoom.",
            "Choosing something to do to a thing that is out of reach walks you to it first.",
            "The slider at the top sets how fast time passes, and Space pauses it. The game \
             pauses by itself when your life is in danger.",
        ],
    ),
    (
        "Water and food",
        &[
            "Drink from the lake. Now and then lake water makes you sick. Water boiled in a \
             birch-bark pot by a fire never does.",
            "Fish with a rod, sink fish traps by the shore, and set snares for hares. Pick \
             bilberries from the middle of July and lingonberries from late August. The inner \
             bark of pine fills a stomach when nothing else does.",
            "Raw meat and fish may make you sick, and go off in a day or two unless it is \
             freezing. Cook them over a fire, or hang them on a drying rack for two days to \
             make them keep for good.",
        ],
    ),
    (
        "Foraging and hunting",
        &[
            "The forest gives different things at different times. Nettles are cut from              the middle of May to the end of July: six make a length of cordage, and four              boil down to greens. Bilberries ripen in July, lingonberries late in August,              and chanterelles come up from August into October.",
            "With a bow and arrows you can hunt from any tree. A hunt takes two hours and              an arrow whether or not it finds a deer, goes better the more you have              trapped and hunted, and best of all on snow, where tracks show. A roe deer              is a heavy load, and butchers into thirty pieces of meat, a hide, and bones.              Dry or smoke what you cannot eat in two days.",
            "A pack on your back lets you carry more, and carry it lightly: eight \
             kilograms more with one woven of birch bark, fifteen with one of leather. \
             Snow slows anyone on foot by a fifth. On skis, which take a log, an axe, and \
             a fire to bend the tips over, it is the fastest going there is.",
        ],
    ),
    (
        "Fire and warmth",
        &[
            "A campfire takes five sticks, a piece of birch bark for tinder, and a fire steel. \
             Rain and wind make it harder to light. A stick burns for a quarter of an hour \
             and a log for two hours.",
            "Stay within reach of the fire to be warmed by it. A lean-to keeps the rain and \
             the wind off, a bough bed makes for better sleep, and a log cabin is warm \
             even in the dead of winter.",
            "Hare pelts sewn into a hat and mittens, boots, and a coat keep you far warmer \
             than the clothes you came in.",
        ],
    ),
    (
        "Making things",
        &[
            "The Crafting tab lists everything you can make. What you carry and what lies on \
             the ground within your reach both count, so heavy things like logs can be \
             piled up where you build.",
            "Cordage is twisted from spruce roots, and most things take some. A stone axe, \
             of two stones, a stick, and cordage, fells trees for logs.",
            "Work set aside is taken up from where you left it.",
        ],
    ),
    (
        "Sleep",
        &[
            "Sleep when you are tired, or you will collapse sooner or later. Nobody falls \
             asleep cold, and the cold wakes a sleeper, so see to your fire first.",
        ],
    ),
    (
        "Other people",
        &[
            "You are not the only one in the forest. Now and then smoke rises where a \
             wanderer has made camp, and the journal tells which way. They stay for a week \
             or two and move on, unless you give them a reason to stay.",
            "Talk with them, once a day counts for the most, and give them food. Someone who \
             has come to trust you can be asked to stay with you. Someone fond enough of you \
             may share your life, and under a roof the two of you may have a child.",
            "The People tab lists everyone you know of. Children grow up, and from twelve \
             they work.",
        ],
    ),
    (
        "A settlement",
        &[
            "You start alone in the wild, or as a newcomer in a hamlet, as you chose. \
             Alone, raise a cairn of ten stones, from the Crafting tab, to found a \
             settlement of your own, and win over the wanderers who camp nearby. Its \
             common store is at hand by the cairn, in storehouses, and at the market: put \
             your load in it, take from it, and make things straight out of it.",
            "The Town tab shows how the place fares: how many it houses and feeds, how \
             content people are, what is in the store, what stands built, and who works \
             at what. Everyone eats out of the store, and those who go hungry leave or \
             starve. Houses are heated with logs, and later with coal, through the frost.",
            "The lake and the forest feed only so many fishers, trappers, and foragers. \
             Beyond a few dozen people it takes fields and pastures. A settlement that is \
             fed, housed, and content draws others to it, the more the larger it is.",
        ],
    ),
    (
        "Trades and buildings",
        &[
            "Most trades take a building to work at, and a building gives work to only \
             so many: three farmers to a field, six miners to a mine, two smiths to a \
             smithy. Whoever leads plans buildings in the Town tab out of the common \
             store, and sets people to trades in the People tab.",
            "Trades hand on to one another. Burners char logs, miners bring up ore, \
             smiths smelt the two into metal and forge tools, and tools in the store make \
             all work yield more until they wear out. Sawyers cut planks, brickmakers \
             fire bricks, glassmakers make panes, and of those the better houses are \
             built. Nothing is made beyond what the settlement can use or sell.",
            "Once a place is built up, a bigger house goes up where the smallest stood: \
             cottages for huts, town houses for cottages, and in the end tenements and \
             apartment blocks.",
        ],
    ),
    (
        "Knowledge and the ages",
        &[
            "The Knowledge tab lists everything there is to work out, age by age, from \
             the Stone Age through bronze, iron, the Middle Ages, print, steam, and \
             electricity to the Modern Age. Each advance takes others first, and opens up \
             buildings, trades, things to make by hand, or laws.",
            "Nothing is worked out by hand. Thinkers do it, and later scholars at \
             libraries and universities, kept by the settlement or hired by you. They get \
             further with a workshop, with writing, with every person who can read, with \
             books in the store, and with trade. It is the work of generations: expect \
             to hand it on to your children.",
        ],
    ),
    (
        "Coin and trade",
        &[
            "With coinage and a market, the store sells and buys. What you take from it \
             costs a fifth over what it is worth, and what you put in it pays a fifth \
             under. A shift at any workplace, from the Surroundings tab, pays half a \
             day's wage, and so does an hour's building. You can owe the market, but \
             people take note.",
            "The treasury raises taxes on what people earn and on what they sell outside. \
             It pays those who build, think, teach, heal, and keep the rolls, and the \
             upkeep of everything but houses. Whoever leads sets the taxes, and can send \
             the merchants for anything the settlement cannot make yet. Merchants, \
             sailors, and transport workers decide how much can be traded in a day.",
        ],
    ),
    (
        "Standing and leading",
        &[
            "Only whoever leads sets people to work, plans buildings, sets taxes, and \
             makes laws. Until that is you, the settlement runs itself, and you live in \
             it as one of its people. When you are chosen, custom keeps running things \
             until you untick it in the Politics tab.",
            "The Politics tab shows who would be chosen if a leader were chosen today. Each \
             grown-up backs whoever they think the most of: you by their regard for you, \
             others by the esteem they are held in. Kin back kin, and people back those who \
             hold what they hold.",
            "Win regard by talking with people, giving them food, bringing goods to the \
             common store, and working on buildings. Speak by the cairn, hold feasts out of \
             your own food, and once writing is known put up notices. With a printing \
             house you can take space in the newspaper, and with a broadcasting house \
             speak over the air, both for coin. A speech reaches sixty people and a \
             notice only those who can read.",
            "Whoever leads can give four offices to people the settlement thinks well \
             of: a marshal for the defence, a treasurer for the taxes, a chancellor for \
             those who think, and an almoner for those in need. Each does some good \
             whoever holds it, and twice as much under someone of the right kind: bold, \
             stern, clever, or kind. Being given an office wins you a friend.",
            "A band and a village choose a leader at the moot each spring. A town, which \
             takes twenty-five people, a hall, and writing, elects one every four years, \
             with a month of campaigning before. With a hundred and twenty people, \
             written law, and a town hall it is chartered as a city under a mayor, and \
             with five hundred people and a power station it is a metropolis.",
        ],
    ),
    (
        "Views and laws",
        &[
            "Say what you hold on four matters in the Politics tab: whether goods are \
             shared, whether to keep to the old ways, who decides, and how strangers are \
             met. Everyone has views of their own, and children take after their parents.",
            "Each matter has laws on either side, more of them as more is known, and each \
             changes how the settlement fares: taxes, trade, crafts, who may settle, and \
             how leaders are chosen, down to a birthright that hands leadership to a \
             leader's eldest child. The leader of a band makes laws alone. Otherwise a \
             law takes half of the grown-ups, or two thirds under an assembly. Every law \
             pleases those it suits and angers the rest.",
        ],
    ),
    (
        "People and their trades",
        &[
            "Everyone is someone. Hover over a name in the People tab to see what they \
             hold, what they think of you, and how they are: diligent or easygoing, clever, \
             kind or stern, hardy or frail, bold. Children take after their parents as \
             often as not.",
            "Everyone gets better at their trade with the years, the diligent twice as \
             fast, and a trade of masters yields three tenths more than one of beginners. \
             Set someone to another trade and half of what they were good at is lost, so \
             think before you move a master smith to the fields. The clever get twice as \
             far as thinkers, scholars, and teachers, the bold are worth two as guards, \
             and the hardy outlive the frail.",
        ],
    ),
    (
        "The wider world",
        &[
            "The World tab lists the other settlements within reach. They grow and learn \
             as yours does. Those who know more than you and are not hostile help your \
             thinkers along just by being there to be watched.",
            "Whoever leads can send an envoy to each once a month: with gifts, to be \
             thought better of; with the offer of a pact, which brings more trade; to ask \
             to be taught, which brings what is being worked out a fifth of the way on; \
             or with every guard there is, to raid them.",
            "Hostile neighbours raid in late summer, once you are twenty-five or more. What \
             your guards, your palisade, and your town wall cannot hold against, they \
             carry off: a fifth of the food, a fifth of the treasury, and some lives. \
             Guards take a barracks, and a barracks takes bronze.",
            "Not every year is like the last. One summer in ten is too dry and one is \
             rich. Now and then something built of wood burns down, less often where \
             there is a well. And where there is food and ale at midsummer, it is kept, \
             and people are the more content for it.",
        ],
    ),
    (
        "Your own fortune",
        &[
            "What you earn is yours. At the bottom of the Town tab you can buy a share in \
             any workplace that stands, one share in each. It pays you a quarter of what \
             those who work there earn, every day, less taxes, for as long as it stands \
             and is manned. Your partner or your child inherits your purse; anyone else \
             who lives on after you does not, and it goes to the treasury.",
            "Coin buys standing as well. Give to the treasury and people think better of \
             you, and large gifts go down in the chronicle. Give to those who think, as \
             their patron, and what is being worked out comes sooner.",
            "The Chronicle tab keeps what the settlement remembers of itself for good: its \
             founding, its leaders, its laws, the first of every kind of building, what \
             was worked out and when, raids, fires, and sicknesses. The People tab lists \
             your kin, the living and the dead.",
        ],
    ),
    (
        "Faith, pastimes, and crime",
        &[
            "People want more than bread. A sauna, a shrine with a priest, and later a              temple, a theatre, a park, a museum, a cinema, and a stadium each make              people more content, as long as the priests and performers they take are              at work. Content people stay, draw others, and think well of whoever leads.",
            "Crime grows where people are idle, crowded, and discontented, and simply with              the size of the place. Thieves take from the store and the treasury, and              nobody is content where crime is rife. Guards, a courthouse with clerks, and              priests keep it down.",
            "Industry has its price. Kilns, brickworks, steelworks, factories, and power \
             stations that burn coal foul the air and the water: more sicken, and people \
             are less content. Waterworks, a park, and in the end clean power help. Once \
             there is a market and twenty-five people, an autumn fair is held every \
             September, which brings a week of trade in a day.",
            "Some winters come early and bite deep, and take half as much wood again. Some springs              the lake comes over its banks and spoils food in the store, less of it where              there is a storehouse.",
        ],
    ),
    (
        "Generations",
        &[
            "Turn on the daily routine by the activity line, and you eat, drink, and sleep \
             by yourself when you have the means. Then time can be run fast, up to a day \
             a second, to see years pass.",
            "Nobody lives for ever. When you die, you can live on as any of those who lived \
             with you and are twelve or older: your child, your partner, or anyone of your \
             settlement. What you built, learned, and knew of the land stays. A child              of yours starts with half of every skill you had. From the age of eight \
             you can apprentice your own children to a trade in the People tab, so that \
             they start at it with years of learning behind them.",
        ],
    ),
    (
        "Saving",
        &[
            "The game is saved when you leave it, and from time to time as you play. Save \
             under a name of your own from the File menu.",
        ],
    ),
];

/// Shows the guide, and returns whether it is to stay open.
pub fn show(ctx: &egui::Context) -> bool {
    let mut close = false;
    let max_height = ctx.content_rect().height() * MAX_HEIGHT_SHARE;

    let modal = egui::Modal::new(egui::Id::unique("help")).show(ctx, |ui| {
        ui.set_width(
            DIALOG_WIDTH
                .min(ctx.content_rect().width() - 48.0)
                .max(120.0),
        );
        ui.heading("How to Play");
        ui.separator();

        egui::ScrollArea::vertical()
            .max_height(max_height)
            .show(ui, |ui| {
                for (title, paragraphs) in CHAPTERS {
                    ui.add_space(6.0);
                    ui.label(RichText::new(title).strong());
                    for paragraph in paragraphs {
                        ui.label(*paragraph);
                        ui.add_space(2.0);
                    }
                }
            });

        ui.add_space(8.0);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            close = ui.button("Close").clicked();
        });
    });

    !(close || modal.should_close())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guide_draws_in_any_window_and_stays_open_until_closed() {
        for (width, height) in [(960.0, 540.0), (300.0, 200.0)] {
            let ctx = egui::Context::default();
            let input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(width, height),
                )),
                ..Default::default()
            };

            let mut open = false;
            ctx.run_ui(input, |ui| open = show(ui.ctx()))
                .drop_without_applying_deltas();
            assert!(open);
        }
    }

    #[test]
    fn every_chapter_has_something_to_say() {
        for (title, paragraphs) in CHAPTERS {
            assert!(!title.is_empty());
            assert!(!paragraphs.is_empty());
        }
    }
}
