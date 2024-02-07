use stabby_abi::{IDiscriminantProvider, IStable};

#[crate::stabby]
/// A trait for key-value maps, allowing to pass hashmaps and the likes accross FFI boundary.
pub trait IMap<K: IStable, V: IStable + IDiscriminantProvider<()>> {
    /// Returns a reference to the value associated to `key`, or None if the key isn't present in the map.
    extern "C" fn get<'a>(&'a self, key: &K) -> crate::option::Option<&'a V>;
    /// Returns a mutable reference to the value associated to `key`, or None if the key isn't present in the map.
    extern "C" fn get_mut<'a>(&'a mut self, key: &K) -> crate::option::Option<&'a mut V>;
    /// Inserts `value`, associated to `key`, returning the previous associated value if it existed.
    extern "C" fn insert(&mut self, key: K, value: V) -> crate::option::Option<V>;
}

#[cfg(feature = "alloc")]
impl<K: IStable + Ord, V: IStable + IDiscriminantProvider<()>> IMap<K, V>
    for std_alloc::collections::BTreeMap<K, V>
{
    extern "C" fn get<'a>(&'a self, key: &K) -> crate::option::Option<&'a V> {
        self.get(key).into()
    }

    extern "C" fn get_mut<'a>(&'a mut self, key: &K) -> crate::option::Option<&'a mut V> {
        self.get_mut(key).into()
    }

    extern "C" fn insert(&mut self, key: K, value: V) -> crate::option::Option<V> {
        self.insert(key, value).into()
    }
}

#[cfg(feature = "std")]
impl<K: IStable + core::hash::Hash + Eq, V: IStable + IDiscriminantProvider<()>> IMap<K, V>
    for std::collections::HashMap<K, V>
{
    extern "C" fn get<'a>(&'a self, key: &K) -> crate::option::Option<&'a V> {
        self.get(key).into()
    }

    extern "C" fn get_mut<'a>(&'a mut self, key: &K) -> crate::option::Option<&'a mut V> {
        self.get_mut(key).into()
    }

    extern "C" fn insert(&mut self, key: K, value: V) -> crate::option::Option<V> {
        self.insert(key, value).into()
    }
}
