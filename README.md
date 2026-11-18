# Lord's Simulator

[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)

GUI-based life simulator built with Rust, focusing on realism and limitless possibilities.

Runs on Linux, Windows, and macOS.

## The game

You are alone in a northern forest by a lake, with a knife, a fire steel, and a little bread.
It is the first of May, and nothing is asked of you but to live.

- **A body to keep alive.** Hunger, thirst, tiredness, body heat, sickness, and health all
  follow from what you do and where you are. Hunger kills in weeks, thirst in days, the cold
  in hours.
- **A sky that turns.** Seasons, weather, and daylight follow the calendar of a place far in
  the north: bright summer nights, a few hours of winter sun, snow on the ground, ice on the
  lake.
- **A forest to live off.** Gather sticks, boughs, bark, roots, and stones, pick berries in
  their season, fell trees, fish, and set snares and fish traps.
- **Things to make.** Cordage, an axe, a fishing rod, pots, fur clothes, a campfire, a
  lean-to, a drying rack, and in the end a log cabin, all from what the forest gives.
- **Other people.** Wanderers camp in the forest. Befriend them, ask them to stay, take a
  partner, and raise children.
- **A settlement of your own.** Found one and give your people work: foragers, fishers,
  trappers, woodcutters, farmers, builders, crafters, and thinkers. Plan huts, longhouses,
  storehouses, workshops, fields, and wells, and work out advances from smoking and pottery
  to farming, ironworking, and writing.
- **Standing and politics.** You start as a newcomer in someone else's hamlet. Earn regard,
  win the moot, and lead. Bands grow into villages and towns, with laws put to the vote,
  views that divide people, speeches, feasts, notices, and elections.
- **Generations.** Run time at up to a day a second while your character keeps to a daily
  routine. When you die, live on as your child, your partner, or anyone of your settlement.
- **A map in pixel art** that changes with the seasons, the snow, and the night.
- **Skills that grow** with use, a journal of what has happened, and saves that carry on from
  where you left.

*Help → How to Play* in the game tells the rest.

## Download

Packages for all three systems are attached to each
[release](https://github.com/thelordman/lords_simulator/releases).

- **Windows:** unzip and run `lords_simulator.exe`.
- **Linux:** unpack the `tar.gz` and run `lords_simulator`.
- **macOS:** unzip `Lords-Simulator-…-macos-universal.zip` and open the app. The `tar.gz` holds the same program.

## Building

With a stable [Rust toolchain](https://rustup.rs) installed (1.95 or newer), on any of the three
systems:

```sh
cargo run --release
```

Linux needs a C linker.
