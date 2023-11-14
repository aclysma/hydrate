/// Default hashmap for hydrate. Opts-out of more expensive secure hash.
pub type HashMap<K, V> = std::collections::HashMap<K, V, ahash::RandomState>;
/// Default hashset for hydrate. Opts-out of more expensive secure hash.
pub type HashSet<T> = std::collections::HashSet<T, ahash::RandomState>;
