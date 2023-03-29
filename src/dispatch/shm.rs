use std::borrow::Borrow;
use std::collections::BTreeMap;
use std::sync::Mutex;

/// Used to share data between macro invokations.
///
/// We use BTreeMap because we're not storing a lot of data.
#[derive(Debug)]
pub struct SharedMemory<K, V>(Mutex<BTreeMap<K, V>>);
unsafe impl<K, V> Sync for SharedMemory<K, V> {}

impl<K, V> SharedMemory<K, V> {
    pub const fn new() -> Self {
        Self(Mutex::new(BTreeMap::new()))
    }

    pub fn insert(&self, key: K, val: V)
    where
        K: Ord,
    {
        if let Ok(ref mut s) = self.0.lock() {
            s.insert(key, val);
        }
    }

    pub fn find<Q>(&self, key: &Q) -> Option<V>
    where
        K: Borrow<Q> + Ord,
        V: Clone,
        Q: Ord,
    {
        if let Ok(s) = self.0.lock() {
            s.get(key).map(Clone::clone)
        } else {
            None
        }
    }
}
