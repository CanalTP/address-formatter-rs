use enum_map::{Enum, EnumMap};
use serde::Serialize;
use strum_macros::{Display, EnumIter, EnumString};

#[derive(Enum, EnumString, Debug, Clone, EnumIter, Copy, Hash, Display, Eq, PartialEq)]
#[strum(serialize_all = "snake_case")]
pub enum Component {
    Attention,
    HouseNumber,
    House,
    Road,
    Village,
    Suburb,
    City,
    County,
    CountyCode,
    Postcode,
    StateDistrict,
    State,
    StateCode,
    Region,
    Island,
    Neighbourhood,
    Country,
    CountryCode,
    Continent,
    Town,
    CityDistrict,
}

impl serde::Serialize for Component {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

#[derive(Debug, Default, Serialize)]
pub struct Address(EnumMap<Component, Option<String>>);

impl std::ops::Deref for Address {
    type Target = EnumMap<Component, Option<String>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for Address {
    fn deref_mut(&mut self) -> &mut EnumMap<Component, Option<String>> {
        &mut self.0
    }
}

impl Address {
    pub fn from_hashmap<'a>(data: impl IntoIterator<Item = (Component, &'a str)>) -> Self {
        let mut a = Self::default();
        for (k, v) in data.into_iter() {
            a[k] = Some(v.to_owned());
        }
        a
    }
}
