//! The pixel art the map is drawn with.
//!
//! Every sprite is written out as rows of letters, one letter for one pixel, and turned
//! into a texture the first time it is drawn.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};

use egui::{Color32, ColorImage, TextureHandle, TextureOptions};
use lords_sim::Item;
use lords_sim::climate::{Conditions, Season};
use lords_sim::people::Bond;
use lords_sim::world::{Kind, PlantSpecies, StructureKind};

/// The side of the tiles the ground and the water are laid with, in pixels, and in meters.
pub const TILE: usize = 32;
pub const TILE_METERS: f32 = 8.0;

const SPRUCE: &[&str] = &[
    ".....kk.....",
    "....kggk....",
    "....kgGk....",
    "...kgGGgk...",
    "...kgGlGk...",
    "..kgGGGGgk..",
    "..kgGlGGgk..",
    ".kgGGGGlGgk.",
    ".kgGlGGGGgk.",
    "kgGGGGGlGGgk",
    "kggGGgGGGggk",
    ".kkkkbbkkkk.",
    "....kbbk....",
    ".....kk.....",
];

const PINE: &[&str] = &[
    "...kkkkkk...",
    "..kpPPPPpk..",
    ".kpPPlPPPpk.",
    "kpPPPPPlPPpk",
    "kpPlPPPPPPpk",
    "kppPPPPlPppk",
    ".kppPPPPppk.",
    "..kkpppkkk..",
    "....kBBk....",
    "....kBbk....",
    "....kBBk....",
    "....kBbk....",
    "....kbbk....",
    ".....kk.....",
];

const BIRCH: &[&str] = &[
    "....kkkk....",
    "..kklllGkk..",
    ".klllYllGGk.",
    ".kllGllllGk.",
    "klllllGlYllk",
    "klGllllllGlk",
    ".kllYlGlllk.",
    ".kGllllGGk..",
    "..kkGwwkkk..",
    "....kwwk....",
    "....kwdk....",
    "....kwwk....",
    "....kdwk....",
    ".....kk.....",
];

const BARE_BIRCH: &[&str] = &[
    "............",
    "....b..b....",
    "..b..b.b..b.",
    "...b.bb.b...",
    ".b..bbbb..b.",
    "..bb.bb.bb..",
    "....bbbb....",
    ".....ww.....",
    "....kwwk....",
    "....kwwk....",
    "....kwdk....",
    "....kwwk....",
    "....kdwk....",
    ".....kk.....",
];

const CHANTERELLE: &[&str] = &[
    "..........",
    "..yy...y..",
    ".yYYy.yYy.",
    "..ty...t..",
    "....yy....",
    ".y.yYYy.y.",
    ".t..tt..t.",
];

const NETTLE: &[&str] = &[
    "..l..l..l.",
    ".lGl.G.lG.",
    "..G.lGl.G.",
    ".lGl.G.lGl",
    "..G.lGl.G.",
    ".lG..G..Gl",
    "..g..g..g.",
];

const BILBERRY: &[&str] = &[
    "..kkkkkk..",
    ".kgGGmGgk.",
    "kgGmGGGmgk",
    "kGGGGmGGGk",
    "kgmGGGGmgk",
    ".kgGGmGgk.",
    "..kkkkkk..",
];

const LINGONBERRY: &[&str] = &[
    "..kkkkkk..",
    ".kGllrlGk.",
    "kGlrlllrGk",
    "kllllrlllk",
    "kGrllllrGk",
    ".kGllrlGk.",
    "..kkkkkk..",
];

const BOULDER: &[&str] = &[
    "...kkkk...",
    ".kksssskk.",
    "kssswsssSk",
    "ksswssssSk",
    "kssssssSSk",
    "kSssssSSdk",
    ".kSSSSddk.",
    "..kkkkkk..",
];

const LOGS: &[&str] = &[
    "..kkkkkk..",
    ".kBtBBtBk.",
    "kBBBkkBBBk",
    "ktBkBtkBtk",
    "kBBkBBkBBk",
    ".kBBkkBBk.",
    "..kkkkkk..",
];

const STONES: &[&str] = &["..kk.kk.", ".kskksk.", "kssSkSsk", "kSSkkSSk", ".kk..kk."];

const BUNDLE: &[&str] = &[
    "..kkkk..", ".kttttk.", "ktBttBtk", "kttttttk", "ktBttBtk", ".kttttk.", "..kkkk..",
];

const CAMPFIRE: &[&str] = &[
    "....F.....",
    "...FfF....",
    "...ffF..F.",
    "..FfwfF...",
    "..FfffF...",
    ".kFFfFFk..",
    "kbkFFFkbk.",
    "kBbkkkbBk.",
    ".kBBbBBk..",
    "..kkkkk...",
];

const FIREPLACE: &[&str] = &[
    ".kkk.kkk..",
    "kSdkkkdSk.",
    "kdkddSkdk.",
    ".kSddddk..",
    "..kkkkk...",
];

const LEAN_TO: &[&str] = &[
    ".....kkkkkkkk.",
    "....kgGgGgGgGk",
    "...kGgGgGgGgGk",
    "..kgGgGgGgGgGk",
    ".kGgGgGgGgGgk.",
    "kgGgGgGgGgGk..",
    "kbkkkkkkkkbk..",
    "kbk......kbk..",
    "kbk......kbk..",
    ".k........k...",
];

const BED: &[&str] = &[
    ".kkkkkkkk.",
    "kgGlGgGlGk",
    "kGgGgGlGgk",
    "kgGlGgGgGk",
    "kGgGgGlGgk",
    ".kkkkkkkk.",
];

const HUT: &[&str] = &[
    ".......kk.......",
    ".....kkBBkk.....",
    "...kkBBttBBkk...",
    ".kkBBttBBttBBkk.",
    "kBBttBBttBBttBBk",
    "kkkBBttBBttBBkkk",
    ".kbkkkBBBBkkkbk.",
    ".kBBBBkkkkBBBBk.",
    ".kbbbbbbbbbbbbk.",
    ".kBBBBBkkBBBBBk.",
    ".kbbbbkddkbbbbk.",
    ".kBBBBkddkBBBBk.",
    ".kbbbbkddkbbbbk.",
    ".kkkkkkkkkkkkkk.",
];

const LONGHOUSE: &[&str] = &[
    "...........kk...........",
    ".......kkkkBBkkkk.......",
    "...kkkkBBttBBttBBkkkk...",
    ".kkBBttBBttBBttBBttBBkk.",
    "kBBttBBttBBttBBttBBttBBk",
    "kkkBBttBBttBBttBBttBBkkk",
    ".kbkkkBBBBkkkkBBBBkkkbk.",
    ".kBBBBkkkkBBBBkkkkBBBBk.",
    ".kbbbbbbbbbbbbbbbbbbbbk.",
    ".kBBkwkBBBBkkBBBBkwkBBk.",
    ".kbbkkkbbbkddkbbbkkkbbk.",
    ".kBBBBBBBBkddkBBBBBBBBk.",
    ".kbbbbbbbbkddkbbbbbbbbk.",
    ".kkkkkkkkkkkkkkkkkkkkkk.",
];

const STOREHOUSE: &[&str] = &[
    "....kkkkkk....",
    "..kkttttttkk..",
    ".kttBttttBttk.",
    "kttttttttttttk",
    "kkkkkkkkkkkkkk",
    ".kBbBbBbBbBbk.",
    ".kbBbBkkBbBbk.",
    ".kBbBbkdkBbBk.",
    ".kkkkkkkkkkkk.",
    "..kb......bk..",
    "..kb......bk..",
    "..kk......kk..",
];

const SMOKEHOUSE: &[&str] = &[
    "......ss..",
    ".....s..s.",
    "....ss....",
    "....kk....",
    "..kkddkk..",
    ".kddSSddk.",
    "kddSSSSddk",
    "kkkkkkkkkk",
    ".kBbBbBbk.",
    ".kbBkkBbk.",
    ".kBbkdbBk.",
    ".kbBkdBbk.",
    ".kkkkkkkk.",
];

const WORKSHOP: &[&str] = &[
    ".kkkkkkkkkkkk.",
    "kBtBtBtBtBtBtk",
    "ktBtBtBtBtBtBk",
    "kkkkkkkkkkkkkk",
    ".kb........bk.",
    ".kb..kkkk..bk.",
    ".kb.kssSSk.bk.",
    ".kb.kkkkkk.bk.",
    ".kb..kb.kb.bk.",
    ".kb..kb.kb.bk.",
    ".kk..kk.kk.kk.",
];

const FIELD: &[&str] = &[
    "kBkBkBkBkBkBkBkB",
    "ByYyyYyyYyyYyyYk",
    "kbbbbbbbbbbbbbbB",
    "ByyYyyYyyYyyYyyk",
    "kYyyYyyYyyYyyYyB",
    "Bbbbbbbbbbbbbbbk",
    "kyYyyYyyYyyYyyYB",
    "ByyYyyYyyYyyYyyk",
    "kbbbbbbbbbbbbbbB",
    "BYyyYyyYyyYyyYyk",
    "kyYyyYyyYyyYyyYB",
    "Bbbbbbbbbbbbbbbk",
    "kyyYyyYyyYyyYyyB",
    "BYyyYyyYyyYyyYyk",
    "kbbbbbbbbbbbbbbB",
    "BkBkBkBkBkBkBkBk",
];

const DOCK: &[&str] = &[
    "kBtBtBtBtBtk",
    "ktBtBtBtBtBk",
    "kBtBtBtBtBtk",
    "kkkkkkkkkkkk",
    ".kb..kb..kb.",
    ".kb..kb..kb.",
    ".uu..uu..uu.",
];

const PASTURE: &[&str] = &[
    "k..k..k..k..k..k",
    "kttkttkttkttkttk",
    "k..............k",
    "k..ww....BB....k",
    "k.wwwk...BBBk..k",
    "k..k.k...k.k...k",
    "k..............k",
    "kttkttkttkttkttk",
    "k..k..k..k..k..k",
];

const COTTAGE: &[&str] = &[
    ".....kkkk.....",
    "...kkttttkk...",
    "..kttttttttk..",
    ".kttttttttttk.",
    "kttttttttttttk",
    "kkkkkkkkkkkkkk",
    ".kBBBBBBBBBBk.",
    ".kBaaBkkBaaBk.",
    ".kBaaBkdBaaBk.",
    ".kBBBBkdBBBBk.",
    ".kBBBBkdBBBBk.",
    ".kkkkkkkkkkkk.",
];

const TOWNHOUSE: &[&str] = &[
    "....kkkk....",
    "...kTTTTk...",
    "..kTTTTTTk..",
    ".kTTTTTTTTk.",
    "kTTTTTTTTTTk",
    "kkkkkkkkkkkk",
    "kRRRRRRRRRRk",
    "kRaaRRRRaaRk",
    "kRaaRRRRaaRk",
    "kRRRRRRRRRRk",
    "kDDDDDDDDDDk",
    "kRaaRkkRaaRk",
    "kRaaRkdRaaRk",
    "kRRRRkdRRRRk",
    "kRRRRkdRRRRk",
    "kkkkkkkkkkkk",
];

const TENEMENT: &[&str] = &[
    "kkkkkkkkkkkkkkkk",
    "kDDDDDDDDDDDDDDk",
    "kRaaRaaRRaaRaaRk",
    "kRaaRaaRRaaRaaRk",
    "kRRRRRRRRRRRRRRk",
    "kRaaRaaRRaaRaaRk",
    "kRaaRaaRRaaRaaRk",
    "kRRRRRRRRRRRRRRk",
    "kRaaRaaRRaaRaaRk",
    "kRaaRaaRRaaRaaRk",
    "kRRRRRRRRRRRRRRk",
    "kRaaRaaRRaaRaaRk",
    "kRaaRaaRRaaRaaRk",
    "kRRRRRRRRRRRRRRk",
    "kRaaRaaRRaaRaaRk",
    "kRaaRaaRRaaRaaRk",
    "kRRRRRRRRRRRRRRk",
    "kRaaRRRkkRRRaaRk",
    "kRRRRRRkdRRRRRRk",
    "kkkkkkkkkkkkkkkk",
];

const APARTMENT: &[&str] = &[
    "kkkkkkkkkkkkkkkk",
    "kQQQQQQQQQQQQQQk",
    "kqaaqaaqqaaqaaqk",
    "kqaaqaaqqaaqaaqk",
    "kqqqqqqqqqqqqqqk",
    "kqaaqaaqqaaqaaqk",
    "kqaaqaaqqaaqaaqk",
    "kqqqqqqqqqqqqqqk",
    "kqaaqaaqqaaqaaqk",
    "kqaaqaaqqaaqaaqk",
    "kqqqqqqqqqqqqqqk",
    "kqaaqaaqqaaqaaqk",
    "kqaaqaaqqaaqaaqk",
    "kqqqqqqqqqqqqqqk",
    "kqaaqaaqqaaqaaqk",
    "kqaaqaaqqaaqaaqk",
    "kqqqqqqqqqqqqqqk",
    "kqaaqaaqqaaqaaqk",
    "kqaaqaaqqaaqaaqk",
    "kqqqqqqqqqqqqqqk",
    "kqaaqaaqqaaqaaqk",
    "kqaaqaaqqaaqaaqk",
    "kqqqqqqqqqqqqqqk",
    "kqaaqqqkkqqqaaqk",
    "kqqqqqqkaqqqqqqk",
    "kkkkkkkkkkkkkkkk",
];

const MILL: &[&str] = &[
    "....kkkkkk......",
    "..kkttttttkk....",
    ".kttttttttttk...",
    "kkkkkkkkkkkkkk..",
    ".kSsSsSsSsSsk...",
    ".kssaasssssskkk.",
    ".kSsaasSsSskBBBk",
    ".kssssssssskBkBk",
    ".kSsSkksSsskBBBk",
    ".kssskdssssskBk.",
    ".kSsSkdSsSsSkkk.",
    ".kkkkkkkkkkkk...",
    "uuuuuuuuuuuuuuuu",
    ".uu..uuu...uu..u",
];

const MINE: &[&str] = &[
    "......SS......",
    "....SSssSS....",
    "..SSssssssSS..",
    ".SssskkkkkssS.",
    "SssskBBBBBkssS",
    "SssskBdddBkssS",
    "SssskBdddBkssS",
    "SssskBdddBkssS",
    "kkkkkkkkkkkkkk",
];

const QUARRY: &[&str] = &[
    "..kkkkkkkkkkkk..",
    ".kSSSSSSSSSSSSk.",
    "kSsssssssssssSSk",
    "kSsSSSSSSSSssSSk",
    "kSsSddddddSssSSk",
    "kSsSSSSSSSSssSSk",
    ".kSSSSSSSSSSSSk.",
    "..kkkkkkkkkkkk..",
];

const MARKET: &[&str] = &[
    ".kkkkkk..kkkkkk.",
    "krwrwrwkkuwuwuwk",
    "kwrwrwrkkwuwuwuk",
    "kkkkkkkkkkkkkkkk",
    ".kb....bkb....bk",
    ".kbyyoobkbGGrrbk",
    ".kbBBBBbkbBBBBbk",
    ".kb....bkb....bk",
    ".kk....kkk....kk",
];

const CIVIC: &[&str] = &[
    "........kk........",
    "......kkwwkk......",
    "....kkwwwwwwkk....",
    "..kkwwwwwwwwwwkk..",
    "kkwwwwwwwwwwwwwwkk",
    "kkkkkkkkkkkkkkkkkk",
    ".kwskwskwskwskwsk.",
    ".kwskwskwskwskwsk.",
    ".kwskwskwskwskwsk.",
    ".kwskwskkkkwskwsk.",
    ".kwskwskddkwskwsk.",
    ".kwskwskddkwskwsk.",
    "kkkkkkkkkkkkkkkkkk",
    "kssssssssssssssssk",
];

const FACTORY: &[&str] = &[
    "...............xx...",
    "..............xx....",
    "..............kkk...",
    "..............kRk...",
    "..k...k...k...kRk...",
    ".kek.kek.kek..kRk...",
    "keeekeeekeeek.kRk...",
    "kkkkkkkkkkkkkkkRkkkk",
    "kRRRRRRRRRRRRRRRRRRk",
    "kRaaRaaRaaRaaRaaRRRk",
    "kRaaRaaRaaRaaRaaRRRk",
    "kRRRRRRRRRRkkRRRRRRk",
    "kRRRRRRRRRRkdRRRRRRk",
    "kkkkkkkkkkkkkkkkkkkk",
];

const POWER: &[&str] = &[
    "...xx........xx.....",
    "....xx........xx....",
    "...kkk.......kkk....",
    "...kqk.......kqk....",
    "...kqk.......kqk....",
    "...kqk.......kqk....",
    "..kkqkk.....kkqkk...",
    "..kqqqk.....kqqqk...",
    "kkkqqqkkkkkkkqqqkkkk",
    "kQQQQQQQQQQQQQQQQQQk",
    "kQyQQyQQyQQyQQyQQyQk",
    "kQQQQQQQQQQQQQQQQQQk",
    "kQyQQyQQkkQQyQQyQQyk",
    "kQQQQQQQkdQQQQQQQQQk",
    "kkkkkkkkkkkkkkkkkkkk",
];

const STATION: &[&str] = &[
    "....kkkkkkkkkkkkkk....",
    "..kkeeeeeeeeeeeeeekk..",
    ".keeeeeeeeeeeeeeeeeek.",
    "kkkkkkkkkkkkkkkkkkkkkk",
    ".kRRaaRRaaRkkRaaRRaaRk",
    ".kRRaaRRaaRkdRaaRRaaRk",
    ".kRRRRRRRRRkdRRRRRRRRk",
    "kkkkkkkkkkkkkkkkkkkkkk",
    "bibibibibibibibibibibi",
    "kkkkkkkkkkkkkkkkkkkkkk",
];

const HARBOUR: &[&str] = &[
    ".........k..........",
    ".........kvv........",
    ".........kvvvv......",
    ".........kvvvvvv....",
    ".........kvvvv......",
    ".........k..........",
    "...kkkkkkkkkkkkkk...",
    "....kBBBBBBBBBBk....",
    "uuuuukkkkkkkkkkuuuuu",
    "kBtBtBtBtBtBtBtBtBtk",
    "kkkkkkkkkkkkkkkkkkkk",
    ".kb...kb....kb...kb.",
];

const AIRPORT: &[&str] = &[
    "..kkk...................",
    "..kak......kkkkkkkk.....",
    "..kak....kkqqqqqqqqkk...",
    "..kqk...kqqqqqqqqqqqqk..",
    "..kqk...kqqkkkkkkkkqqk..",
    "..kqk...kqqkddddddkqqk..",
    "kkkkkkkkkkkkkkkkkkkkkkkk",
    "dddddddddddddddddddddddd",
    "dwwddwwddwwddwwddwwddwwd",
    "dddddddddddddddddddddddd",
];

const MAST: &[&str] = &[
    "....r.....",
    "....k.....",
    "...kkk....",
    "....k.....",
    "...kkk....",
    "....k.....",
    "..kkkkk...",
    "....k.....",
    ".kkkkkkk..",
    "....k.....",
    "kkkkkkkkkk",
    "kqqqqqqqqk",
    "kqaaqqaaqk",
    "kqaaqqaaqk",
    "kqqqqqqqqk",
    "kqqqkkqqqk",
    "kqqqkdqqqk",
    "kkkkkkkkkk",
];

const OFFICE: &[&str] = &[
    "kkkkkkkkkkkkkk",
    "kQQQQQQQQQQQQk",
    "kaaqaaqaaqaaqk",
    "kaaqaaqaaqaaqk",
    "kqqqqqqqqqqqqk",
    "kaaqaaqaaqaaqk",
    "kaaqaaqaaqaaqk",
    "kqqqqqqqqqqqqk",
    "kaaqaaqaaqaaqk",
    "kaaqaaqaaqaaqk",
    "kqqqqqqqqqqqqk",
    "kqqqqkkkkqqqqk",
    "kqqqqkaakqqqqk",
    "kkkkkkkkkkkkkk",
];

const TANK: &[&str] = &[
    "...kkkkkk...",
    ".kksssssskk.",
    "ksssssssSSSk",
    "ksssssssSSSk",
    "kssuuussSSSk",
    "ksssssssSSSk",
    "ksssssssSSSk",
    ".kksssssskk.",
    "..kkkkkkkk..",
    "..k.k..k.k..",
];

const CHARCOAL_KILN: &[&str] = &[
    "....xx....",
    "...xx.....",
    "...kkkk...",
    "..kbbbbk..",
    ".kbBbbBbk.",
    ".kbbFFbbk.",
    "kbbBFFBbbk",
    "kkkkkkkkkk",
];

const PALISADE: &[&str] = &[
    ".k..k..k..k..k..",
    "kBkkBkkBkkBkkBk.",
    "kBkkBkkBkkBkkBk.",
    "kbkkbkkbkkbkkbk.",
    "kBkkBkkBkkBkkBk.",
    "kbkkbkkbkkbkkbk.",
    "kkkkkkkkkkkkkkk.",
];

const WALL: &[&str] = &[
    "kkk.kkk.kkk.kkk.",
    "kSk.kSk.kSk.kSk.",
    "kSkkkSkkkSkkkSkk",
    "kSsSSsSSsSSsSSsk",
    "ksSSsSSkkSsSSsSk",
    "kSsSSsSkdkSSsSSk",
    "ksSSsSSkdkSsSSsk",
    "kkkkkkkkkkkkkkkk",
];

const SHRINE: &[&str] = &[
    "....kk....",
    "...kyyk...",
    "..kkkkkk..",
    "..ksSssk..",
    "..kSkkSk..",
    "..kskdsk..",
    ".kkkkkkkk.",
    ".kssssssk.",
];

const PARK: &[&str] = &[
    ".GG....lll....GG",
    "GlGG..lGGGl..GGl",
    ".GG...lGGl....GG",
    ".b.....bb.....b.",
    "ttttttttttttttt.",
    ".r.y...kBBk..y.r",
    "ttttttttttttttt.",
    ".GG....lll....GG",
];

const WELL: &[&str] = &[
    "..kkkkkk..",
    ".kBBBBBBk.",
    ".kbkkkkbk.",
    "..kb..bk..",
    "..kb..bk..",
    ".kkskkskk.",
    "kssSuuSssk",
    "ksSuuuuSsk",
    "kSssSSssSk",
    ".kkkkkkkk.",
];

const CAIRN: &[&str] = &[
    "....kk....",
    "...kssk...",
    "...kSsk...",
    "..kksSkk..",
    "..kssSSk..",
    ".kkSssSkk.",
    ".ksSSssSk.",
    "kssSsSSssk",
    "kSSssSsSdk",
    ".kkkkkkkk.",
];

const SITE: &[&str] = &[
    ".kk........kk.",
    ".kBkkkkkkkkBk.",
    ".kBtttttttkBk.",
    ".kBk......kBk.",
    ".kBk......kBk.",
    ".kBk......kBk.",
    ".kBkkkkkkkkBk.",
    ".kBtttttttkBk.",
    ".kk........kk.",
];

const SNARE: &[&str] = &["..kk..", ".k..k.", ".k..k.", "..kk..", "..kb..", "..kb.."];

const SPRUNG_SNARE: &[&str] = &["..kk..", ".kwwk.", "kwwwwk", ".kwwk.", "..kb..", "..kb.."];

const FISH_TRAP: &[&str] = &[
    ".kkkkkk.", "kBtBtBtk", "ktBtBtBk", "kBtBtBtk", "ktBtBtBk", ".kkkkkk.",
];

const DRYING_RACK: &[&str] = &[
    "kk........kk",
    "kBkkkkkkkkBk",
    "kBttttttttBk",
    "kBkrkrkrkrBk",
    "kBkrkrkrkrBk",
    "kBk.k.k.k.Bk",
    "kBk.......Bk",
    "kBk.......Bk",
    "kk........kk",
];

/// Someone standing, whose clothes are the letter `C`.
const FIGURE: &[&str] = &[
    "..kkkk..", ".kHHHHk.", ".kHhhHk.", ".khhhhk.", "..khhk..", ".kCCCCk.", "kCCCCCCk", "khCCCChk",
    ".kCCCCk.", ".kCkkCk.", ".kbkkbk.", ".kk..kk.",
];

/// Something there is a picture of.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Sprite {
    Thing(Kind),
    Player,
    Ground,
    Water,
}

/// What the pictures are made to fit: the time of the year, and the weather.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Dress {
    Spring,
    Summer,
    Autumn,
    Winter,
    /// Under snow, or under ice for water, whatever the time of the year.
    Snow,
}

impl Dress {
    pub fn of(conditions: &Conditions) -> Self {
        if conditions.snow {
            return Self::Snow;
        }
        match conditions.season {
            Season::Spring => Self::Spring,
            Season::Summer => Self::Summer,
            Season::Autumn => Self::Autumn,
            Season::Winter => Self::Winter,
        }
    }

    fn is_wintry(self) -> bool {
        matches!(self, Self::Winter | Self::Snow)
    }
}

/// The texture of each sprite in each dress, as it is and flipped left to right.
type Textures = HashMap<(Sprite, Dress, bool), TextureHandle>;

/// The textures of the sprites drawn so far.
#[derive(Clone, Default)]
pub struct Art {
    textures: Arc<Mutex<Textures>>,
}

impl Art {
    /// Returns the art kept with `ctx`, which is empty until something is drawn.
    pub fn of(ctx: &egui::Context) -> Self {
        ctx.data_mut(|data| {
            data.get_temp_mut_or_default::<Self>(egui::Id::unique("map_art"))
                .clone()
        })
    }

    /// Returns the texture of `sprite` in `dress`, flipped left to right if `flipped`.
    pub fn texture(
        &self,
        ctx: &egui::Context,
        sprite: Sprite,
        dress: Dress,
        flipped: bool,
    ) -> TextureHandle {
        // Only what changes with the dress is kept once for each.
        let dress = if changes_with_dress(sprite) {
            dress
        } else {
            Dress::Summer
        };
        let mut textures = self.textures.lock().unwrap_or_else(PoisonError::into_inner);

        textures
            .entry((sprite, dress, flipped))
            .or_insert_with(|| {
                let image = picture(sprite, dress, flipped);
                let options = match sprite {
                    // Smoothed when shrunk, so that the ground does not shimmer from afar.
                    Sprite::Ground | Sprite::Water => TextureOptions {
                        minification: egui::TextureFilter::Linear,
                        ..TextureOptions::NEAREST_REPEAT
                    },
                    Sprite::Thing(_) | Sprite::Player => TextureOptions::NEAREST,
                };
                ctx.load_texture(format!("{sprite:?} {dress:?} {flipped}"), image, options)
            })
            .clone()
    }
}

fn changes_with_dress(sprite: Sprite) -> bool {
    match sprite {
        Sprite::Ground | Sprite::Water => true,
        Sprite::Thing(Kind::Plant(_) | Kind::Rock) => true,
        Sprite::Thing(Kind::Structure(StructureKind::Field)) => true,
        Sprite::Thing(_) | Sprite::Player => false,
    }
}

/// Returns the picture of `sprite` in `dress`.
pub fn picture(sprite: Sprite, dress: Dress, flipped: bool) -> ColorImage {
    let mut pixels = match sprite {
        Sprite::Ground => ground(dress),
        Sprite::Water => water(dress),
        Sprite::Player => drawn(FIGURE, Some(Color32::from_rgb(244, 236, 214))),
        Sprite::Thing(kind) => thing(kind, dress),
    };
    if flipped {
        for row in &mut pixels {
            row.reverse();
        }
    }

    let size = [pixels.first().map_or(0, Vec::len), pixels.len()];
    let bytes: Vec<u8> = pixels
        .into_iter()
        .flatten()
        .flat_map(|pixel| pixel.to_srgba_unmultiplied())
        .collect();
    ColorImage::from_rgba_unmultiplied(size, &bytes)
}

fn thing(kind: Kind, dress: Dress) -> Vec<Vec<Color32>> {
    let rows = match kind {
        Kind::Plant(PlantSpecies::Spruce) => SPRUCE,
        Kind::Plant(PlantSpecies::Pine) => PINE,
        Kind::Plant(PlantSpecies::Birch) if dress.is_wintry() => BARE_BIRCH,
        Kind::Plant(PlantSpecies::Birch) => BIRCH,
        Kind::Plant(PlantSpecies::Bilberry) => BILBERRY,
        Kind::Plant(PlantSpecies::Lingonberry) => LINGONBERRY,
        Kind::Plant(PlantSpecies::Chanterelle) => CHANTERELLE,
        Kind::Plant(PlantSpecies::Nettle) => NETTLE,
        Kind::Rock | Kind::Lake => BOULDER,
        Kind::Pile(Item::Log | Item::Stick) => LOGS,
        Kind::Pile(Item::Stone) => STONES,
        Kind::Pile(_) => BUNDLE,
        Kind::Structure(structure) => match structure {
            StructureKind::Campfire => CAMPFIRE,
            StructureKind::Fireplace => FIREPLACE,
            StructureKind::LeanTo => LEAN_TO,
            StructureKind::Bed => BED,
            StructureKind::Cabin | StructureKind::Hut => HUT,
            StructureKind::Longhouse => LONGHOUSE,
            StructureKind::Snare => SNARE,
            StructureKind::SprungSnare => SPRUNG_SNARE,
            StructureKind::FishTrap => FISH_TRAP,
            StructureKind::DryingRack => DRYING_RACK,
            StructureKind::Cairn => CAIRN,
            StructureKind::Site => SITE,
            StructureKind::Storehouse => STOREHOUSE,
            StructureKind::Smokehouse => SMOKEHOUSE,
            StructureKind::Workshop => WORKSHOP,
            StructureKind::Field => FIELD,
            StructureKind::Well => WELL,
            StructureKind::Hall => LONGHOUSE,
            StructureKind::Kiln => SMOKEHOUSE,
            StructureKind::Smithy => WORKSHOP,
            StructureKind::Dock => DOCK,
            StructureKind::Sauna => SMOKEHOUSE,
            StructureKind::Shrine => SHRINE,
            StructureKind::Temple | StructureKind::Theatre | StructureKind::Museum => CIVIC,
            StructureKind::Park => PARK,
            StructureKind::Cinema => OFFICE,
            StructureKind::Stadium => QUARRY,
            StructureKind::Palisade => PALISADE,
            StructureKind::Wall => WALL,
            StructureKind::Barracks => LONGHOUSE,
            StructureKind::Pasture => PASTURE,
            StructureKind::Brewery => COTTAGE,
            StructureKind::Tavern => COTTAGE,
            StructureKind::Quarry => QUARRY,
            StructureKind::Mine => MINE,
            StructureKind::CharcoalKiln => CHARCOAL_KILN,
            StructureKind::Weavery => COTTAGE,
            StructureKind::Bathhouse => COTTAGE,
            StructureKind::Market => MARKET,
            StructureKind::Courthouse => CIVIC,
            StructureKind::Infirmary => COTTAGE,
            StructureKind::Harbour => HARBOUR,
            StructureKind::School => COTTAGE,
            StructureKind::Mill => MILL,
            StructureKind::Sawmill => MILL,
            StructureKind::Cottage => COTTAGE,
            StructureKind::Brickworks => FACTORY,
            StructureKind::Glassworks => FACTORY,
            StructureKind::Guildhall => CIVIC,
            StructureKind::Library => CIVIC,
            StructureKind::Townhouse => TOWNHOUSE,
            StructureKind::TownHall => CIVIC,
            StructureKind::Bank => CIVIC,
            StructureKind::PrintingHouse => FACTORY,
            StructureKind::University => CIVIC,
            StructureKind::Waterworks => TANK,
            StructureKind::Steelworks => FACTORY,
            StructureKind::Factory => FACTORY,
            StructureKind::TextileMill => FACTORY,
            StructureKind::Tenement => TENEMENT,
            StructureKind::RailwayStation => STATION,
            StructureKind::Cannery => FACTORY,
            StructureKind::PowerStation => POWER,
            StructureKind::TelegraphOffice => MAST,
            StructureKind::Hospital => OFFICE,
            StructureKind::ColdStore => TANK,
            StructureKind::MachineDepot => WORKSHOP,
            StructureKind::BroadcastingHouse => MAST,
            StructureKind::ApartmentBlock => APARTMENT,
            StructureKind::Airport => AIRPORT,
            StructureKind::Institute => OFFICE,
            StructureKind::ComputingCentre => OFFICE,
        },
        Kind::Person(bond) => {
            return drawn(
                FIGURE,
                Some(match bond {
                    Bond::Stranger => Color32::from_rgb(150, 150, 162),
                    Bond::Friend => Color32::from_rgb(70, 164, 150),
                    Bond::Companion => Color32::from_rgb(84, 124, 204),
                    Bond::Partner => Color32::from_rgb(224, 104, 136),
                    Bond::Child => Color32::from_rgb(232, 192, 84),
                }),
            );
        }
    };

    let mut pixels = drawn(rows, None);
    for (from, to) in repainted(kind) {
        recolor(&mut pixels, color(*from), color(*to));
    }
    match (kind, dress) {
        (Kind::Plant(PlantSpecies::Birch), Dress::Autumn) => {
            recolor(&mut pixels, color('l'), color('y'));
            recolor(&mut pixels, color('G'), color('o'));
        }
        (Kind::Plant(PlantSpecies::Bilberry | PlantSpecies::Lingonberry), Dress::Autumn) => {
            recolor(&mut pixels, color('G'), color('o'));
        }
        (Kind::Structure(StructureKind::Field), Dress::Spring) => {
            recolor(&mut pixels, color('y'), color('l'));
            recolor(&mut pixels, color('Y'), color('G'));
        }
        (Kind::Structure(StructureKind::Field), Dress::Winter | Dress::Snow) => {
            recolor(&mut pixels, color('y'), color('w'));
            recolor(&mut pixels, color('Y'), color('n'));
        }
        (Kind::Plant(species), Dress::Snow) if species != PlantSpecies::Birch => {
            snow_cap(&mut pixels);
        }
        (Kind::Rock, Dress::Snow) => snow_cap(&mut pixels),
        _ => {}
    }
    pixels
}

/// Returns the colors that are painted over in the picture a building shares with
/// others, so that each can be told apart: the color painted over, and the color it is
/// painted with.
fn repainted(kind: Kind) -> &'static [(char, char)] {
    match kind {
        Kind::Structure(StructureKind::Barracks) => &[('t', 'S'), ('B', 'd')],
        Kind::Structure(StructureKind::Temple) => &[('w', 'Y')],
        Kind::Structure(StructureKind::Theatre) => &[('w', 'r')],
        Kind::Structure(StructureKind::Museum) => &[('s', 'u')],
        Kind::Structure(StructureKind::Cinema) => &[('q', 'm')],
        Kind::Structure(StructureKind::Stadium) => &[('S', 'q'), ('d', 'G'), ('s', 'l')],
        Kind::Structure(StructureKind::Brewery) => &[('t', 'o')],
        Kind::Structure(StructureKind::Tavern) => &[('t', 'r')],
        Kind::Structure(StructureKind::Weavery) => &[('t', 'u')],
        Kind::Structure(StructureKind::Bathhouse) => &[('B', 's'), ('t', 'S')],
        Kind::Structure(StructureKind::Infirmary) => &[('B', 'w'), ('t', 'r')],
        Kind::Structure(StructureKind::School) => &[('t', 'G')],
        Kind::Structure(StructureKind::Sawmill) => &[('s', 'B'), ('S', 'b')],
        Kind::Structure(StructureKind::Brickworks) => &[('e', 'T')],
        Kind::Structure(StructureKind::Glassworks) => &[('R', 's'), ('e', 'u')],
        Kind::Structure(StructureKind::Guildhall) => &[('w', 't'), ('s', 'B')],
        Kind::Structure(StructureKind::Library) => &[('w', 't')],
        Kind::Structure(StructureKind::TownHall) => &[('s', 'y')],
        Kind::Structure(StructureKind::Bank) => &[('w', 's'), ('s', 'S')],
        Kind::Structure(StructureKind::PrintingHouse) => &[('R', 't'), ('e', 'b')],
        Kind::Structure(StructureKind::University) => &[('w', 'R'), ('s', 'D')],
        Kind::Structure(StructureKind::Steelworks) => &[('R', 'S'), ('e', 'd')],
        Kind::Structure(StructureKind::TextileMill) => &[('R', 'q'), ('e', 'u')],
        Kind::Structure(StructureKind::Cannery) => &[('R', 'B'), ('e', 'S')],
        Kind::Structure(StructureKind::TelegraphOffice) => &[('q', 'R')],
        Kind::Structure(StructureKind::Hospital) => &[('q', 'w'), ('Q', 'r')],
        Kind::Structure(StructureKind::ColdStore) => &[('s', 'w'), ('S', 'n')],
        Kind::Structure(StructureKind::MachineDepot) => &[('B', 'S'), ('t', 's')],
        Kind::Structure(StructureKind::ComputingCentre) => &[('q', 'S'), ('Q', 'd')],
        _ => &[],
    }
}

/// Returns the pixels of the picture written out in `rows`, the letter `C` standing for
/// `clothes`.
fn drawn(rows: &[&str], clothes: Option<Color32>) -> Vec<Vec<Color32>> {
    let width = rows.iter().map(|row| row.len()).max().unwrap_or(0);

    rows.iter()
        .map(|row| {
            let mut pixels: Vec<Color32> = row
                .chars()
                .map(|letter| match (letter, clothes) {
                    ('C', Some(clothes)) => clothes,
                    _ => color(letter),
                })
                .collect();
            pixels.resize(width, Color32::TRANSPARENT);
            pixels
        })
        .collect()
}

fn recolor(pixels: &mut [Vec<Color32>], from: Color32, to: Color32) {
    for pixel in pixels.iter_mut().flatten() {
        if *pixel == from {
            *pixel = to;
        }
    }
}

/// Whitens the topmost pixels inside the outline of each column, as snow lying on it.
fn snow_cap(pixels: &mut [Vec<Color32>]) {
    let outline = color('k');
    let width = pixels.first().map_or(0, Vec::len);

    for column in 0..width {
        let mut covered = 0;
        for row in pixels.iter_mut() {
            let pixel = &mut row[column];
            if *pixel == Color32::TRANSPARENT || *pixel == outline {
                continue;
            }
            *pixel = if covered == 0 { color('w') } else { color('n') };
            covered += 1;
            if covered == 2 {
                break;
            }
        }
    }
}

/// Returns the color a letter of a picture stands for.
fn color(letter: char) -> Color32 {
    match letter {
        'k' => Color32::from_rgb(40, 34, 44),
        'g' => Color32::from_rgb(34, 88, 62),
        'G' => Color32::from_rgb(54, 126, 74),
        'l' => Color32::from_rgb(120, 176, 84),
        'p' => Color32::from_rgb(62, 104, 62),
        'P' => Color32::from_rgb(110, 148, 72),
        'y' => Color32::from_rgb(222, 186, 70),
        'Y' => Color32::from_rgb(242, 222, 124),
        'o' => Color32::from_rgb(222, 128, 52),
        'b' => Color32::from_rgb(92, 62, 44),
        'B' => Color32::from_rgb(142, 98, 60),
        't' => Color32::from_rgb(198, 162, 112),
        'w' => Color32::from_rgb(240, 240, 232),
        'n' => Color32::from_rgb(206, 218, 234),
        's' => Color32::from_rgb(176, 178, 188),
        'S' => Color32::from_rgb(124, 126, 138),
        'd' => Color32::from_rgb(78, 80, 92),
        'r' => Color32::from_rgb(204, 62, 66),
        'u' => Color32::from_rgb(72, 110, 204),
        'm' => Color32::from_rgb(86, 78, 172),
        'h' => Color32::from_rgb(232, 192, 152),
        'H' => Color32::from_rgb(112, 76, 50),
        'f' => Color32::from_rgb(255, 222, 96),
        'F' => Color32::from_rgb(250, 140, 44),
        'R' => Color32::from_rgb(170, 84, 68),
        'D' => Color32::from_rgb(128, 60, 52),
        'T' => Color32::from_rgb(196, 96, 60),
        'q' => Color32::from_rgb(196, 196, 188),
        'Q' => Color32::from_rgb(150, 150, 146),
        'a' => Color32::from_rgb(150, 204, 226),
        'e' => Color32::from_rgb(74, 80, 104),
        'i' => Color32::from_rgb(132, 138, 150),
        'v' => Color32::from_rgb(236, 228, 204),
        'x' => Color32::from_rgb(214, 214, 214),
        _ => Color32::TRANSPARENT,
    }
}

/// Returns a number from 0 to 99 that is always the same for the same pixel of a tile.
fn speckle(x: usize, y: usize, salt: u32) -> u32 {
    let mut mixed = (x as u32)
        .wrapping_mul(0x9E37_79B1)
        .wrapping_add((y as u32).wrapping_mul(0x85EB_CA77))
        .wrapping_add(salt.wrapping_mul(0xC2B2_AE3D));
    mixed ^= mixed >> 15;
    mixed = mixed.wrapping_mul(0x2C1B_3C6D);
    mixed ^= mixed >> 12;
    mixed % 100
}

/// Returns the tile the ground is laid with.
fn ground(dress: Dress) -> Vec<Vec<Color32>> {
    let rgb = Color32::from_rgb;
    let (base, dark, light, fleck) = match dress {
        Dress::Spring => (
            rgb(104, 150, 82),
            rgb(88, 134, 72),
            rgb(128, 170, 92),
            rgb(240, 226, 130),
        ),
        Dress::Summer => (
            rgb(84, 136, 72),
            rgb(70, 120, 64),
            rgb(104, 154, 80),
            rgb(236, 236, 220),
        ),
        Dress::Autumn => (
            rgb(134, 132, 72),
            rgb(116, 112, 62),
            rgb(156, 150, 80),
            rgb(206, 110, 52),
        ),
        Dress::Winter => (
            rgb(124, 122, 92),
            rgb(108, 104, 82),
            rgb(142, 140, 104),
            rgb(208, 214, 222),
        ),
        Dress::Snow => (
            rgb(232, 236, 240),
            rgb(208, 218, 232),
            rgb(246, 248, 250),
            rgb(190, 204, 224),
        ),
    };

    (0..TILE)
        .map(|y| {
            (0..TILE)
                .map(|x| match speckle(x, y, 1) {
                    0..12 => dark,
                    12..22 => light,
                    22 if speckle(x, y, 2) < 40 => fleck,
                    _ => base,
                })
                .collect()
        })
        .collect()
}

/// Returns the tile water is laid with, which is ice under [`Dress::Snow`].
fn water(dress: Dress) -> Vec<Vec<Color32>> {
    let rgb = Color32::from_rgb;
    let (base, crest, depth) = if dress == Dress::Snow {
        (rgb(190, 214, 234), rgb(232, 242, 250), rgb(160, 190, 216))
    } else {
        (rgb(62, 118, 178), rgb(116, 168, 214), rgb(50, 100, 160))
    };

    (0..TILE)
        .map(|y| {
            (0..TILE)
                .map(|x| {
                    let along = (x + (y / 8) * 13) % 16;
                    if y % 8 == 2 && along < 5 {
                        crest
                    } else if y % 8 == 6 && (along + 8) % 16 < 3 {
                        depth
                    } else {
                        base
                    }
                })
                .collect()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const DRESSES: [Dress; 5] = [
        Dress::Spring,
        Dress::Summer,
        Dress::Autumn,
        Dress::Winter,
        Dress::Snow,
    ];

    fn kinds() -> Vec<Kind> {
        let structures = [
            StructureKind::Campfire,
            StructureKind::Fireplace,
            StructureKind::LeanTo,
            StructureKind::Bed,
            StructureKind::Cabin,
            StructureKind::Snare,
            StructureKind::SprungSnare,
            StructureKind::FishTrap,
            StructureKind::DryingRack,
            StructureKind::Cairn,
            StructureKind::Site,
            StructureKind::Hut,
            StructureKind::Longhouse,
            StructureKind::Storehouse,
            StructureKind::Smokehouse,
            StructureKind::Workshop,
            StructureKind::Field,
            StructureKind::Well,
            StructureKind::Hall,
            StructureKind::Kiln,
            StructureKind::Dock,
            StructureKind::Smithy,
        ];
        let buildings = lords_sim::settlement::BUILDINGS
            .iter()
            .map(|plan| plan.kind)
            .filter(|kind| !structures.contains(kind));
        let structures: Vec<StructureKind> = structures.into_iter().chain(buildings).collect();
        let plants = [
            PlantSpecies::Spruce,
            PlantSpecies::Pine,
            PlantSpecies::Birch,
            PlantSpecies::Bilberry,
            PlantSpecies::Lingonberry,
            PlantSpecies::Chanterelle,
            PlantSpecies::Nettle,
        ];
        let bonds = [
            Bond::Stranger,
            Bond::Friend,
            Bond::Companion,
            Bond::Partner,
            Bond::Child,
        ];

        let mut kinds = vec![Kind::Rock, Kind::Pile(Item::Log), Kind::Pile(Item::Stone)];
        kinds.push(Kind::Pile(Item::RyeBread));
        kinds.extend(structures.into_iter().map(Kind::Structure));
        kinds.extend(plants.map(Kind::Plant));
        kinds.extend(bonds.map(Kind::Person));
        kinds
    }

    #[test]
    fn there_is_a_picture_of_everything_in_every_dress() {
        for kind in kinds() {
            for dress in DRESSES {
                let image = picture(Sprite::Thing(kind), dress, false);

                assert!(image.width() >= 5 && image.height() >= 5, "{kind:?}");
                assert!(
                    image.pixels.iter().any(|pixel| pixel.a() > 0),
                    "{kind:?} in {dress:?} is blank"
                );
            }
        }
    }

    #[test]
    fn pictures_are_written_out_with_rows_of_one_width_and_known_letters() {
        let sprites = [
            SPRUCE,
            PINE,
            BIRCH,
            BARE_BIRCH,
            BILBERRY,
            LINGONBERRY,
            BOULDER,
            LOGS,
            STONES,
            BUNDLE,
            CAMPFIRE,
            FIREPLACE,
            LEAN_TO,
            BED,
            HUT,
            LONGHOUSE,
            STOREHOUSE,
            SMOKEHOUSE,
            WORKSHOP,
            FIELD,
            WELL,
            DOCK,
            CAIRN,
            SITE,
            SNARE,
            SPRUNG_SNARE,
            FISH_TRAP,
            DRYING_RACK,
            FIGURE,
        ];

        for rows in sprites {
            let width = rows[0].len();
            for row in rows {
                assert_eq!(row.len(), width, "{row}");
                for letter in row.chars() {
                    let known = letter == '.' || letter == 'C' || color(letter).a() > 0;
                    assert!(known, "{letter} in {row}");
                }
            }
        }
    }

    #[test]
    fn tiles_are_square_and_change_with_the_dress() {
        for sprite in [Sprite::Ground, Sprite::Water] {
            let summer = picture(sprite, Dress::Summer, false);
            let snow = picture(sprite, Dress::Snow, false);

            assert_eq!(summer.size, [TILE, TILE]);
            assert!(summer.pixels.iter().all(|pixel| pixel.a() == 255));
            assert_ne!(summer.pixels, snow.pixels);
        }
    }

    #[test]
    fn birch_turns_yellow_then_bare_and_spruce_wears_snow() {
        let birch = |dress| {
            picture(
                Sprite::Thing(Kind::Plant(PlantSpecies::Birch)),
                dress,
                false,
            )
        };
        let spruce = |dress| {
            picture(
                Sprite::Thing(Kind::Plant(PlantSpecies::Spruce)),
                dress,
                false,
            )
        };
        let has = |image: &ColorImage, letter| image.pixels.contains(&color(letter));

        assert!(has(&birch(Dress::Summer), 'l') && !has(&birch(Dress::Summer), 'y'));
        assert!(has(&birch(Dress::Autumn), 'y') && !has(&birch(Dress::Autumn), 'l'));
        assert!(!has(&birch(Dress::Winter), 'l') && !has(&birch(Dress::Winter), 'y'));
        assert!(!has(&spruce(Dress::Winter), 'w'));
        assert!(has(&spruce(Dress::Snow), 'w'));
    }

    #[test]
    fn flipped_picture_is_the_mirror_image() {
        let fire = Sprite::Thing(Kind::Structure(StructureKind::Campfire));
        let one = picture(fire, Dress::Summer, false);
        let other = picture(fire, Dress::Summer, true);

        assert_ne!(one.pixels, other.pixels);
        assert_eq!(one.pixels[0], other.pixels[one.width() - 1]);
    }

    #[test]
    fn textures_are_made_once_and_kept_with_the_context() {
        let ctx = egui::Context::default();
        let art = Art::of(&ctx);
        let rock = Sprite::Thing(Kind::Rock);

        let first = art.texture(&ctx, rock, Dress::Summer, false);
        let again = Art::of(&ctx).texture(&ctx, rock, Dress::Summer, false);
        let cabin = Sprite::Thing(Kind::Structure(StructureKind::Cabin));
        let in_summer = art.texture(&ctx, cabin, Dress::Summer, false);
        let in_winter = art.texture(&ctx, cabin, Dress::Winter, false);

        assert_eq!(first.id(), again.id());
        assert_eq!(in_summer.id(), in_winter.id());
        assert_ne!(first.id(), in_summer.id());
    }
}
