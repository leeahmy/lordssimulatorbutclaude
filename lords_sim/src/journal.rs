//! The record of what has happened to the player character.

use serde::{Deserialize, Serialize};

use crate::time::Timestamp;

/// How many of the latest entries are kept.
pub const LIMIT: usize = 300;

/// Whether what an entry tells of is welcome.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Tone {
    Plain,
    Good,
    Bad,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    pub time: Timestamp,
    pub tone: Tone,
    pub text: String,
}

/// The latest things that have happened, oldest first.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Journal {
    entries: Vec<Entry>,
}

impl Journal {
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// Writes an entry, unless it says just what the latest one does.
    pub(crate) fn write(&mut self, time: Timestamp, tone: Tone, text: String) {
        if self.entries.last().is_some_and(|last| last.text == text) {
            return;
        }

        if self.entries.len() >= LIMIT {
            let excess = self.entries.len() + 1 - LIMIT;
            self.entries.drain(..excess);
        }
        self.entries.push(Entry { time, tone, text });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_latest_entries_are_kept() {
        let mut journal = Journal::default();

        for second in 0..LIMIT as u64 + 5 {
            journal.write(
                Timestamp::from_seconds(second),
                Tone::Plain,
                format!("Entry {second}"),
            );
        }

        assert_eq!(journal.entries().len(), LIMIT);
        assert_eq!(journal.entries()[0].text, "Entry 5");
        assert_eq!(
            journal.entries()[LIMIT - 1].text,
            format!("Entry {}", LIMIT + 4)
        );
    }

    #[test]
    fn saying_the_same_thing_twice_in_a_row_is_written_once() {
        let mut journal = Journal::default();
        let write = |journal: &mut Journal, text: &str| {
            journal.write(Timestamp::EPOCH, Tone::Bad, text.to_owned());
        };

        write(&mut journal, "You need an axe.");
        write(&mut journal, "You need an axe.");
        write(&mut journal, "You need a knife.");
        write(&mut journal, "You need an axe.");

        assert_eq!(journal.entries().len(), 3);
    }
}
