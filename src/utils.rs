// use std::collections::{BTreeSet, BTreeMap};



// trait Utils<T> {
//     fn get_or_insert_with<F, U>(&mut self, key: T, f: F) -> &mut U where F: FnOnce() -> U;
// }

// impl<T, U> Utils<T> for BTreeMap<T, BTreeSet<U>> {
//     fn get_or_insert_with<F, R>(&mut self, key: T, f: F) -> &mut R where F: FnOnce() -> R {
//         self.get_mut(&key).get_or_insert_with(f)
//     }
// }