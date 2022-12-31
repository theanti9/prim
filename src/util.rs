use std::{
    collections::{HashMap, HashSet},
    hash::BuildHasherDefault,
};

use hashers::fx_hash::FxHasher;

/// A HashMap using the [`FxHasher`] for faster but non-cryptographically safe hashing.
pub(crate) type FxHashMap<K, V> = HashMap<K, V, BuildHasherDefault<FxHasher>>;

/// A HashSet using the [`FxHasher`] for faster but non-cryptographically safe hashing.
pub(crate) type FxHashSet<T> = HashSet<T, BuildHasherDefault<FxHasher>>;
