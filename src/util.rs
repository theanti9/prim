use std::{
    collections::{HashMap, HashSet},
    hash::BuildHasherDefault,
};

use hashers::fx_hash::FxHasher;

pub type FxHashMap<K, V> = HashMap<K, V, BuildHasherDefault<FxHasher>>;

pub type FxHashSet<T> = HashSet<T, BuildHasherDefault<FxHasher>>;
