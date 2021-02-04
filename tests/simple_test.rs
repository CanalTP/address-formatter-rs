#[macro_use]
extern crate maplit;
use address_formatter::{Component, Formatter, Place};

#[test]
pub fn basic_test() {
    let formatter = Formatter::default();

    let mut addr = Place::default();
    addr[Component::City] = Some("Toulouse".to_owned());
    addr[Component::Country] = Some("France".to_owned());
    addr[Component::CountryCode] = Some("FR".to_owned());
    addr[Component::County] = Some("Toulouse".to_owned());
    addr[Component::HouseNumber] = Some("17".to_owned());
    addr[Component::Neighbourhood] = Some("Lafourguette".to_owned());
    addr[Component::Postcode] = Some("31000".to_owned());
    addr[Component::Road] = Some("Rue du Médecin-Colonel Calbairac".to_owned());
    addr[Component::State] = Some("Midi-Pyrénées".to_owned());
    addr[Component::Suburb] = Some("Toulouse Ouest".to_owned());

    assert_eq!(
        formatter.format(addr).unwrap(),
        r#"17 Rue du Médecin-Colonel Calbairac
31000 Toulouse
France
"#
        .to_owned()
    )
}

#[test]
pub fn easier_init_test() {
    use Component::*;
    let formatter = Formatter::default();

    let data = hashmap!(
        City => "Toulouse",
        Country => "France",
        CountryCode => "FR",
        County => "Toulouse",
        HouseNumber => "17",
        Neighbourhood => "Lafourguette",
        Postcode => "31000",
        Road => "Rue du Médecin-Colonel Calbairac",
        State => "Midi-Pyrénées",
        Suburb => "Toulouse Ouest",
    );

    assert_eq!(
        formatter.format(data).unwrap(),
        r#"17 Rue du Médecin-Colonel Calbairac
31000 Toulouse
France
"#
        .to_owned()
    )
}

#[test]
pub fn empty_address() {
    let formatter = Formatter::default();
    let addr = Place::default();
    assert_eq!(formatter.format(addr).unwrap(), "\n".to_owned())
}

#[test]
pub fn address_builder() {
    let formatter = Formatter::default();
    let addr_builder = address_formatter::PlaceBuilder::default();
    let data = [
        ("building", "Mairie (bureaux administratifs)"),
        ("city", "Papeete"),
        (
            "country",
            "Polynésie française, Îles du Vent (eaux territoriales)",
        ),
        ("country_code", "fr"),
        ("county", "Îles du Vent"),
        ("postcode", "98714"),
        ("road", "Rue des Remparts"),
        ("state", "French Polynesia"),
    ];

    let addr = addr_builder.build_place(data.iter().map(|(k, v)| (*k, v.to_string())));

    assert_eq!(
        formatter.format(addr).unwrap(),
        r#"Mairie (bureaux administratifs)
Rue des Remparts
98714 Papeete
Polynésie française
"#
        .to_owned()
    )
}

#[test]
fn use_of_singleton() {
    assert_eq!(
        address_formatter::FORMATTER
            .format(hashmap!(
                address_formatter::Component::City => "Toulouse",
                address_formatter::Component::Country => "France",
                address_formatter::Component::CountryCode => "FR",
                address_formatter::Component::Road => "Rue du Médecin-Colonel Calbairac",
            ))
            .unwrap(),
        r#"Rue du Médecin-Colonel Calbairac
Toulouse
France
"#
        .to_owned()
    )
}
