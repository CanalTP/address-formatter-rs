use serde::Serialize;

#[derive(Default, Debug, Eq, PartialEq, Serialize)]
pub struct Address {
    pub attention: Option<String>,
    pub house_number: Option<String>,
    pub house: Option<String>,
    pub road: Option<String>,
    pub village: Option<String>,
    pub suburb: Option<String>,
    pub city: Option<String>,
    pub county: Option<String>,
    pub postcode: Option<String>,
    pub state_district: Option<String>,
    pub state: Option<String>,
    pub region: Option<String>,
    pub island: Option<String>,
    pub neighbourhood: Option<String>,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub continent: Option<String>,
}
