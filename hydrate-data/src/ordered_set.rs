use hydrate_base::hashing::HashSet;
use std::hash::Hash;

#[derive(Clone, Default)]
pub struct OrderedSet<T: Eq + PartialEq + Hash + Clone> {
    vec: Vec<T>,
    // the set is just a lookup, the vec is the real authority
    set: HashSet<T>,
}

impl<'a, T: Eq + PartialEq + Hash + Clone> IntoIterator for &'a OrderedSet<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T: std::fmt::Debug + Eq + PartialEq + Hash + Clone> std::fmt::Debug for OrderedSet<T> {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        f.debug_struct("OrderedSet")
            .field("vec", &self.vec)
            // Don't include the set because it's redundant
            .finish()
    }
}

impl<T: Eq + PartialEq + Hash + Clone> OrderedSet<T> {
    pub fn iter(&self) -> std::slice::Iter<T> {
        self.vec.iter()
    }

    pub fn contains(
        &self,
        value: &T,
    ) -> bool {
        self.set.contains(value)
    }

    // Returns true if insert is "successful". Otherwise it's false if it already existed
    pub fn try_insert_at_end(
        &mut self,
        value: T,
    ) -> bool {
        let is_newly_inserted = self.set.insert(value.clone());
        if is_newly_inserted {
            self.vec.push(value);
        }

        is_newly_inserted
    }

    pub fn remove(
        &mut self,
        value: &T,
    ) -> bool {
        let removed = self.set.remove(value);
        if removed {
            self.vec
                .remove(self.vec.iter().position(|x| x == value).unwrap());
        }

        removed
    }

    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }
}
