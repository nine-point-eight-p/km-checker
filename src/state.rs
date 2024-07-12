use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize};
use std::{collections::HashMap, hash::Hash};

/// Common Kernel State Type. With matches function and serde support
pub trait AbstractState: DeserializeOwned + Serialize {
    fn matches(&self, other: &Self) -> bool;
}

/// Not Checked Fileds
#[derive(Debug)]
pub struct Unmatched<T>(pub T);

impl<'de, T> Deserialize<'de> for Unmatched<T>
where
    T: Default,
{
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Unmatched(T::default()))
    }
}

impl<T> Serialize for Unmatched<T>
where
    T: Default,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_none()
    }
}

impl<'a, T> AbstractState for Unmatched<T>
where
    T: Default,
{
    fn matches(&self, _other: &Self) -> bool {
        true
    }
}

/// Common Data Type, Checked for Equality
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Value<T>(pub T);

impl<'a, T> AbstractState for Value<T>
where
    T: PartialEq + DeserializeOwned + Serialize,
{
    /// Values match if they are equal
    fn matches(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

/// Ordered List of Values
#[derive(Serialize, Deserialize, Debug)]
pub struct ValueList<T>(pub Vec<Value<T>>);

impl<'a, T> AbstractState for ValueList<T>
where
    T: PartialEq + DeserializeOwned + Serialize,
{
    fn matches(&self, other: &Self) -> bool {
        if self.0.len() != other.0.len() {
            return false;
        }
        self.0.iter().zip(other.0.iter()).all(|(a, b)| a.matches(b))
    }
}

/// Unordered Set of Values
#[derive(Serialize, Deserialize, Debug)]
pub struct ValueSet<T>(pub Vec<Value<T>>)
where
    T: PartialEq;

impl<'a, T> AbstractState for ValueSet<T>
where
    T: PartialEq + DeserializeOwned + Serialize,
{
    fn matches(&self, other: &Self) -> bool {
        if self.0.len() != other.0.len() {
            return false;
        }
        self.0.iter().any(|a| other.0.iter().any(|b| a.matches(b)))
    }
}

/// Common Identifier. Not checked for equality
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct Ident<T>(pub T);

impl<'a, T> AbstractState for Ident<T>
where
    T: DeserializeOwned + Serialize,
{
    /// Single Identifier always matches
    fn matches(&self, _other: &Self) -> bool {
        return true;
    }
}

/// Ordered List of Identifiers
#[derive(Debug, Deserialize, Serialize)]
pub struct IdentList<T>(pub Vec<Ident<T>>)
where
    T: Hash + Eq;

impl<'a, T> AbstractState for IdentList<T>
where
    T: Hash + Eq + DeserializeOwned + Serialize,
{
    fn matches(&self, other: &Self) -> bool {
        if self.0.len() != other.0.len() {
            return false;
        }
        map_ident(&self.0) == map_ident(&other.0)
    }
}

/// Unordered Set of Identifiers
#[derive(Debug, Deserialize, Serialize)]
pub struct IdentSet<T>(pub Vec<Ident<T>>)
where
    T: Hash + Eq;

impl<'a, T> AbstractState for IdentSet<T>
where
    T: Hash + Eq + DeserializeOwned + Serialize,
{
    fn matches(&self, other: &Self) -> bool {
        if self.0.len() != other.0.len() {
            return false;
        }
        let mut self_mapped = map_ident(&self.0);
        let mut other_mapped = map_ident(&other.0);
        self_mapped.sort();
        other_mapped.sort();
        self_mapped == other_mapped
    }
}

fn map_ident<T>(list: &Vec<Ident<T>>) -> Vec<usize>
where
    T: Hash + Eq,
{
    let mut map = HashMap::new();
    list.iter().for_each(|e| {
        if !map.contains_key(&e.0) {
            map.insert(&e.0, map.len());
        }
    });
    let mut mapped = Vec::new();
    for i in 0..list.len() {
        mapped.push(*map.get(&list[i].0).unwrap());
    }
    mapped
}
