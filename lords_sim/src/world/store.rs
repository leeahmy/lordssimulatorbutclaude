use std::fmt::{self, Formatter};
use std::marker::PhantomData;
use std::sync::Arc;

use serde::de::{SeqAccess, Visitor};
use serde::ser::SerializeSeq;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

const CHUNK_LEN: usize = 256;

/// A list whose clones share their items until one of them changes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Store<T> {
    chunks: Vec<Arc<Vec<T>>>,
}

impl<T> Default for Store<T> {
    fn default() -> Self {
        Self { chunks: Vec::new() }
    }
}

impl<T> Store<T> {
    pub fn len(&self) -> usize {
        match self.chunks.split_last() {
            Some((last, full)) => full.len() * CHUNK_LEN + last.len(),
            None => 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }

    pub fn get(&self, index: u32) -> Option<&T> {
        let index = index as usize;
        self.chunks.get(index / CHUNK_LEN)?.get(index % CHUNK_LEN)
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.chunks.iter().flat_map(|chunk| chunk.iter())
    }
}

impl<T: Clone> Store<T> {
    /// Adds `item` and returns its index, which it keeps for good.
    pub fn push(&mut self, item: T) -> u32 {
        let index = self.len();
        if index.is_multiple_of(CHUNK_LEN) {
            self.chunks.push(Arc::new(Vec::with_capacity(CHUNK_LEN)));
        }
        if let Some(last) = self.chunks.last_mut() {
            Arc::make_mut(last).push(item);
        }

        u32::try_from(index).expect("a store holds no more items than fit its indices")
    }

    pub fn get_mut(&mut self, index: u32) -> Option<&mut T> {
        let index = index as usize;
        let chunk = self.chunks.get_mut(index / CHUNK_LEN)?;
        if index % CHUNK_LEN >= chunk.len() {
            return None;
        }

        Arc::make_mut(chunk).get_mut(index % CHUNK_LEN)
    }
}

impl<T: Clone> FromIterator<T> for Store<T> {
    fn from_iter<I: IntoIterator<Item = T>>(items: I) -> Self {
        let mut store = Self::default();
        for item in items {
            store.push(item);
        }
        store
    }
}

/// A store is written as one sequence of its items, whatever chunks they are kept in.
impl<T: Serialize> Serialize for Store<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut sequence = serializer.serialize_seq(Some(self.len()))?;
        for item in self.iter() {
            sequence.serialize_element(item)?;
        }
        sequence.end()
    }
}

impl<'de, T: Deserialize<'de> + Clone> Deserialize<'de> for Store<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Items<T>(PhantomData<T>);

        impl<'de, T: Deserialize<'de> + Clone> Visitor<'de> for Items<T> {
            type Value = Store<T>;

            fn expecting(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
                formatter.write_str("a sequence of items")
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut sequence: A) -> Result<Store<T>, A::Error> {
                let mut store = Store::default();
                while let Some(item) = sequence.next_element()? {
                    store.push(item);
                }
                Ok(store)
            }
        }

        deserializer.deserialize_seq(Items(PhantomData))
    }
}

/// A set of numbers, kept as a bit for each, whose clones share what they hold until one
/// of them changes.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Bits {
    words: Store<u64>,
}

impl Bits {
    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }

    pub fn has(&self, number: u32) -> bool {
        self.words
            .get(number / u64::BITS)
            .is_some_and(|word| word & (1 << (number % u64::BITS)) != 0)
    }

    pub fn mark(&mut self, number: u32) {
        if self.has(number) {
            return;
        }

        while self.words.len() <= (number / u64::BITS) as usize {
            self.words.push(0);
        }
        if let Some(word) = self.words.get_mut(number / u64::BITS) {
            *word |= 1 << (number % u64::BITS);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn numbers(count: u32) -> Store<u32> {
        (0..count).collect()
    }

    #[test]
    fn items_keep_the_index_they_were_given() {
        let mut store = Store::default();
        assert!(store.is_empty());

        for number in 0..1_000 {
            assert_eq!(store.push(number), number);
        }

        assert_eq!(store.len(), 1_000);
        assert_eq!(store.get(0), Some(&0));
        assert_eq!(store.get(255), Some(&255));
        assert_eq!(store.get(256), Some(&256));
        assert_eq!(store.get(999), Some(&999));
        assert_eq!(store.get(1_000), None);
        assert!(store.iter().copied().eq(0..1_000));
    }

    #[test]
    fn clone_shares_chunks_until_one_is_changed() {
        let mut store = numbers(600);
        let clone = store.clone();

        *store.get_mut(300).expect("the item is there") = 7;

        assert!(Arc::ptr_eq(&store.chunks[0], &clone.chunks[0]));
        assert!(!Arc::ptr_eq(&store.chunks[1], &clone.chunks[1]));
        assert!(Arc::ptr_eq(&store.chunks[2], &clone.chunks[2]));
        assert_eq!(store.get(300), Some(&7));
        assert_eq!(clone.get(300), Some(&300));
    }

    #[test]
    fn clone_is_left_alone_by_later_additions() {
        let mut store = numbers(300);
        let clone = store.clone();

        store.push(300);

        assert_eq!(store.len(), 301);
        assert_eq!(clone.len(), 300);
        assert_eq!(clone.get(300), None);
    }

    #[test]
    fn there_is_nothing_to_change_past_the_end() {
        let mut store = numbers(300);

        assert!(store.get_mut(299).is_some());
        assert!(store.get_mut(300).is_none());
        assert!(store.get_mut(10_000).is_none());
    }

    #[test]
    fn store_is_written_as_one_flat_sequence() {
        let store = numbers(600);

        let written = ron::to_string(&store).expect("the store encodes");
        let plain = ron::to_string(&(0..600).collect::<Vec<u32>>()).expect("the list encodes");
        assert_eq!(written, plain);

        let read: Store<u32> = ron::from_str(&written).expect("the store decodes");
        assert_eq!(read, store);
    }
}
