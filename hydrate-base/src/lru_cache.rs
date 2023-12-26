use std::hash::Hash;
use crate::hashing::HashMap;

#[derive(Copy, Clone)]
struct LruCacheNode {
    next: u32,
    previous: u32,
}

pub struct LruCache<K, V> {
    // Doubly linked list with u32::MAX for "null" and using indices instead of pointers
    lru_list_head: u32,
    lru_list_tail: u32,
    lru_list: Vec<LruCacheNode>,

    // Slots that line up with the doubly linked list
    lru_list_pairs: Vec<Option<(K, V)>>,

    // Lookup for the index a key is stored at
    lookup: HashMap<K, u32>,
}

impl<K: Clone + PartialEq + Eq + Hash, V> LruCache<K, V> {
    pub fn new(size: u32) -> LruCache<K, V> {
        assert!(size > 2);
        let mut lru_list = vec![LruCacheNode { next: 0, previous: 0}; size as usize];
        lru_list[0].previous = u32::MAX;
        lru_list[0].next = 1;
        for i in 1..(size-1) {
            lru_list[i as usize].previous = i - 1;
            lru_list[i as usize].next = i + 1;
        }
        lru_list[size as usize - 1].previous = size - 2;
        lru_list[size as usize - 1].next = u32::MAX;

        let mut lru_list_pairs = Vec::with_capacity(size as usize);
        for _ in 0..size {
            lru_list_pairs.push(None);
        }

        let lookup = HashMap::default();

        LruCache {
            lru_list_head: 0,
            lru_list_tail: size - 1,
            lru_list,
            lru_list_pairs,
            lookup,
        }
    }

    // For debug, can throw in to try and find when state is invalid
    /*
    fn check_list(&self) {
        let mut iter = self.lru_list_head;
        let mut count = 0;
        while self.lru_list[iter as usize].next != u32::MAX {
            let next_index = self.lru_list[iter as usize].next;
            assert_eq!(self.lru_list[next_index as usize].previous, iter);
            iter = self.lru_list[iter as usize].next;
            count += 1;
        }

        assert_eq!(count, self.lru_list.len() - 1);
    }
    */

    pub fn pairs(&self) -> &Vec<Option<(K, V)>> {
        &self.lru_list_pairs
    }

    pub fn pairs_mut(&mut self) -> &mut Vec<Option<(K, V)>> {
        &mut self.lru_list_pairs
    }

    fn move_to_front(&mut self, node_index: u32) {
        //self.check_list();
        let node = self.lru_list[node_index as usize];

        if node_index == self.lru_list_head {
            // Do nothing if already at head
            assert_eq!(node.previous, u32::MAX);
            assert_ne!(node.next, u32::MAX);
            return;
        }

        if node_index == self.lru_list_tail {
            // If we are the tail, make the node previous to us the new tail
            assert_eq!(node.next, u32::MAX);
            assert_ne!(node.previous, u32::MAX);
            self.lru_list_tail = node.previous;
        }

        // splice this node out of the list.
        assert_ne!(node.previous, u32::MAX);
        self.lru_list[node.previous as usize].next = node.next;
        if node.next != u32::MAX {
            self.lru_list[node.next as usize].previous = node.previous;
        }

        // Make this node the new head
        assert_eq!(self.lru_list[self.lru_list_head as usize].previous, u32::MAX);
        self.lru_list[self.lru_list_head as usize].previous = node_index;
        self.lru_list[node_index as usize].previous = u32::MAX;
        self.lru_list[node_index as usize].next = self.lru_list_head;
        self.lru_list_head = node_index;

        //self.check_list();
    }

    fn move_to_back(&mut self, node_index: u32) {
        //self.check_list();
        let node = self.lru_list[node_index as usize];

        if node_index == self.lru_list_tail {
            // Do nothing if we are already the tail
            assert_eq!(node.next, u32::MAX);
            assert_ne!(node.previous, u32::MAX);
            return;
        }

        if node_index == self.lru_list_head {
            // If we are the head, make the node next/after us the new head
            assert_eq!(node.previous, u32::MAX);
            assert_ne!(node.next, u32::MAX);
            self.lru_list_head = node.next;
        }


        // splice this node out of the list.
        if node.previous != u32::MAX {
            self.lru_list[node.previous as usize].next = node.next;
        }
        assert_ne!(node.next, u32::MAX);
        self.lru_list[node.next as usize].previous = node.previous;

        // Make this node the new tail
        assert_eq!(self.lru_list[self.lru_list_tail as usize].next, u32::MAX);
        self.lru_list[self.lru_list_tail as usize].next = node_index;
        self.lru_list[node_index as usize].previous = self.lru_list_tail;
        self.lru_list[node_index as usize].next = u32::MAX;
        self.lru_list_tail = node_index;

        //self.check_list();
    }

    pub fn get(&mut self, k: &K, mark_as_recently_used: bool) -> Option<&V> {
        if let Some(&node_index) = self.lookup.get(k) {
            if mark_as_recently_used {
                // move node to head
                self.move_to_front(node_index);
            }
            // return the value
            self.lru_list_pairs[node_index as usize].as_ref().map(|(_, v)| v)
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, k: &K, mark_as_recently_used: bool) -> Option<&mut V> {
        if let Some(&node_index) = self.lookup.get(k) {
            if mark_as_recently_used {
                // move node to head
                self.move_to_front(node_index);
            }
            // return the value
            self.lru_list_pairs[node_index as usize].as_mut().map(|(_, v)| v)
        } else {
            None
        }
    }

    pub fn insert(&mut self, k: K, v: V) {
        if let Some(key_to_remove) = self.lru_list_pairs[self.lru_list_tail as usize].as_ref().map(|(k, _)| k).cloned() {
            self.remove(&key_to_remove);
        }

        // remove tail element if it exists
        let node_index = self.lru_list_tail;
        if let Some((k, _)) = &self.lru_list_pairs[node_index as usize] {
            self.lookup.remove(k);
        }

        self.move_to_front(self.lru_list_tail);
        self.lookup.insert(k.clone(), node_index);
        self.lru_list_pairs[node_index as usize] = Some((k, v));
    }

    pub fn remove(&mut self, k: &K) -> Option<V> {
        if let Some(&node_index) = self.lookup.get(k) {
            // move node to tail
            self.move_to_back(node_index);
            // return the value
            let v = self.lru_list_pairs[node_index as usize].take().map(|(_, v)| v);
            self.lookup.remove(k);
            v
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn check_lru_gets_full() {
        let mut lru_cache = LruCache::new(3);
        lru_cache.insert(0, 0);
        lru_cache.insert(1, 1);
        lru_cache.insert(2, 2);

        // All should be present
        assert!(lru_cache.get(&0).is_some());
        assert!(lru_cache.get(&1).is_some());
        assert!(lru_cache.get(&2).is_some());

        // The oldest one should be bumped, and the new one should be present
        lru_cache.insert(3, 3);
        assert!(lru_cache.get(&0).is_none());
        assert!(lru_cache.get(&1).is_some());
        assert!(lru_cache.get(&2).is_some());
        assert!(lru_cache.get(&3).is_some());
    }

    #[test]
    fn check_lru_deletes_least_recently_used() {
        let mut lru_cache = LruCache::new(3);
        lru_cache.insert(0, 0);
        lru_cache.insert(1, 1);
        lru_cache.insert(2, 2);

        // All should be present
        assert!(lru_cache.get(&0).is_some());
        assert!(lru_cache.get(&1).is_some());
        assert!(lru_cache.get(&2).is_some());

        // Touch the oldest, preventing it from being removed
        lru_cache.get(&0);

        lru_cache.insert(3, 3);
        assert!(lru_cache.get(&0).is_some());
        assert!(lru_cache.get(&1).is_none());
        assert!(lru_cache.get(&2).is_some());
        assert!(lru_cache.get(&3).is_some());
    }

    #[test]
    fn check_remove() {
        let mut lru_cache = LruCache::new(3);
        lru_cache.insert(0, 0);
        lru_cache.insert(1, 1);
        lru_cache.insert(2, 2);

        // All should be present
        assert!(lru_cache.get(&0).is_some());
        assert!(lru_cache.get(&1).is_some());
        assert!(lru_cache.get(&2).is_some());

        // Touch the oldest, preventing it from being removed
        lru_cache.remove(&0);
        lru_cache.remove(&2);
        lru_cache.remove(&1);

        lru_cache.insert(3, 3);
        assert!(lru_cache.get(&0).is_none());
        assert!(lru_cache.get(&1).is_none());
        assert!(lru_cache.get(&2).is_none());
        assert!(lru_cache.get(&3).is_some());
    }
}
