#![allow(dead_code)]

use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Lookup<K, V>
where
    K: std::hash::Hash + Eq + Sized,
{
    pub access: hashbrown::HashMap<K, V, ahash::RandomState>,
}

impl<K, V> Lookup<K, V>
where
    K: std::hash::Hash + Eq + Sized,
{
    pub fn new() -> Self {
        Self {
            access: hashbrown::HashMap::with_hasher(ahash::RandomState::new()),
        }
    }

    /// Returns the number of elements in the internal map.
    pub fn length(&self) -> usize {
        self.access.len()
    }

    /// An iterator visiting all key-value pairs in arbitrary order.
    /// The iterator element type is `(&'a K, &'a V)`.
    pub fn iter(&self) -> hashbrown::hash_map::Iter<K, V> {
        self.access.iter()
    }

    /// An iterator visiting all key-value pairs in arbitrary order,
    /// with mutable references to the values.
    /// The iterator element type is `(&'a K, &'a mut V)`.
    pub fn iter_mut(&mut self) -> hashbrown::hash_map::IterMut<K, V> {
        self.access.iter_mut()
    }

    /// An iterator visiting all values mutably in arbitrary order.
    /// The iterator element type is `&'a mut V`.
    pub fn values_mut(&mut self) -> hashbrown::hash_map::ValuesMut<K, V> {
        self.access.values_mut()
    }

    /// Returns `true` if the map contains no elements.
    pub fn is_empty(&self) -> bool {
        self.access.is_empty()
    }

    /// Inserts a key-value pair into internal map.
    ///
    /// If the map did not have this key present, [`None`] is returned.
    ///
    /// If the map did have this key present, the value is updated, and the old
    /// value is returned. The key is not updated, though; this matters for
    /// types that can be `==` without being identical.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.access.insert(key, value)
    }

    /// An iterator visiting all keys in arbitrary order.
    /// The iterator element type is `&'a K`.
    pub fn keys(&self) -> hashbrown::hash_map::Keys<K, V> {
        self.access.keys()
    }

    /// An iterator visiting all values in arbitrary order.
    /// The iterator element type is `&'a V`.
    pub fn values(&self) -> hashbrown::hash_map::Values<K, V> {
        self.access.values()
    }

    /// Clears the map, removing all key-value pairs. Keeps the allocated memory
    /// for reuse.
    pub fn clear(&mut self) {
        self.access.clear();
    }

    /// Inserts all new key-values from the iterator and replaces values with existing
    /// keys with new values returned from the iterator.
    pub fn extend<T>(&mut self, other: T)
    where
        T: IntoIterator<Item = (K, V)>,
        K: std::hash::Hash + Eq + Sized,
        V: std::clone::Clone,
    {
        self.access.extend(other);
    }

    /// Removes a key from the internal map, returning the value at the key if the key
    /// was previously in the map. Keeps the allocated memory for reuse.
    pub fn remove<Q>(&mut self, key: &Q)
    where
        K: std::borrow::Borrow<Q>,
        Q: std::hash::Hash + Eq + Sized,
    {
        self.access.remove(key);
    }

    /// Returns `true` if the map contains a value for the specified key.
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: std::borrow::Borrow<Q>,
        Q: std::hash::Hash + Eq + ?Sized,
    {
        self.access.contains_key(key)
    }

    /// Returns a reference to the value corresponding to the key.
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: std::borrow::Borrow<Q>,
        Q: std::hash::Hash + Eq + ?Sized,
    {
        self.access.get(key)
    }

    /// Returns a mutable reference to the value corresponding to the key.
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: std::borrow::Borrow<Q>,
        Q: std::hash::Hash + Eq + ?Sized,
    {
        self.access.get_mut(key)
    }
}

impl<K, V> From<hashbrown::HashMap<K, V>> for Lookup<K, V>
where
    K: std::hash::Hash + Eq + Sized,
{
    fn from(value: hashbrown::HashMap<K, V>) -> Self {
        let access = hashbrown::HashMap::with_capacity_and_hasher(value.len(), ahash::RandomState::new());
        Self { access }
    }
}

impl<K, V> From<Vec<(K, V)>> for Lookup<K, V>
where
    K: std::hash::Hash + Eq + Sized,
{
    fn from(value: Vec<(K, V)>) -> Self {
        let mut map = hashbrown::HashMap::with_capacity_and_hasher(value.len(), ahash::RandomState::new());
        for (key, value) in value {
            map.insert(key, value);
        }
        Lookup { access: map }
    }
}

impl<K, V> FromIterator<(K, V)> for Lookup<K, V>
where
    K: std::hash::Hash + Eq + Sized,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let iterator = iter.into_iter();
        let possible_size = iterator.size_hint().1.unwrap_or(10);
        let mut map = hashbrown::HashMap::with_capacity_and_hasher(possible_size, ahash::RandomState::new());
        for (key, value) in iterator {
            map.insert(key, value);
        }
        Lookup { access: map }
    }
}

impl<K, V> IntoIterator for Lookup<K, V>
where
    K: std::hash::Hash + Eq + Sized,
{
    type Item = (K, V);
    type IntoIter = hashbrown::hash_map::IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.access.into_iter()
    }
}

impl From<Vec<crate::core::profile::Instance>> for Lookup<String, crate::core::profile::Instance> {
    fn from(value: Vec<crate::core::profile::Instance>) -> Self {
        let mut map = hashbrown::HashMap::with_capacity_and_hasher(value.len(), ahash::RandomState::new());
        for instance in value {
            map.insert(instance.name.clone(), instance);
        }
        Lookup { access: map }
    }
}

impl<K, V> Serialize for Lookup<K, V>
where
    K: std::hash::Hash + Serialize + Eq,
    V: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.access
            .iter()
            .collect::<Vec<(&K, &V)>>()
            .serialize(serializer)
    }
}

impl<'de, K, V> Deserialize<'de> for Lookup<K, V>
where
    K: std::hash::Hash + Deserialize<'de> + Eq + Clone,
    V: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let items = Vec::<(K, V)>::deserialize(deserializer)?;
        Ok(Lookup::from(items))
    }
}
