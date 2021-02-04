use crate::formatter::{
    CountryCode, Formatter, NewComponent, PlaceBuilder, ReplaceRule, Replacement, Rules, Template,
    Templates,
};
use crate::Component;
use failure::{format_err, Error};
use std::collections::HashMap;
use std::str::FromStr;

pub fn read_configuration() -> Formatter {
    // read all the opencage configuration
    // let opencage_dir = include_dir!("./address-formatting/conf");
    let templates_file = include_str!("../address-formatting/conf/countries/worldwide.yaml");

    let raw_templates = yaml_rust::YamlLoader::load_from_str(templates_file)
        .expect("impossible to read worldwide.yaml file");
    let default_template = build_template(&raw_templates[0]["default"]["address_template"])
        .expect("no default address_template provided");
    let fallback_template = build_template(&raw_templates[0]["default"]["fallback_template"])
        .expect("no fallback address_template provided");

    // some countries uses the same rules as other countries (with some slight changes)
    // they are marked as `use_country: another_country_code`
    // we store them separatly first, to be able to create fully built templates
    let mut overrided_countries = HashMap::new();

    let mut fallback_templates_by_country = HashMap::new();
    let mut rules_by_country = HashMap::new();
    let mut templates_by_country: HashMap<CountryCode, Template> = raw_templates[0]
        .as_hash()
        .unwrap()
        .iter()
        .filter_map(|(k, v)| {
            k.as_str()
                .and_then(|k| CountryCode::from_str(k).ok())
                .map(|c| (c, v))
        })
        .filter_map(|(country_code, v)| {
            if let Ok(fallback_template) = build_template(&v["fallback_template"]) {
                fallback_templates_by_country.insert(country_code.clone(), fallback_template);
            }
            if let Some(parent_country) = v["use_country"]
                .as_str()
                .and_then(|k| CountryCode::from_str(k).ok())
            {
                // we store it for later processing
                overrided_countries.insert(country_code, (parent_country, v.clone()));
                None
            } else {
                let replace_rules = read_replace(&v["replace"]);
                let post_format_replace_rules = read_replace(&v["postformat_replace"])
                    .into_iter()
                    .map(|r| match r {
                        ReplaceRule::All(r) => r,
                        _ => panic!("postformat rules cannot be applied on only one element"),
                    })
                    .collect();

                let template = build_template(&v["address_template"]).unwrap_or_else(|err| {
                    panic!(
                        "Err {}: no address_template found for country {}",
                        err, country_code
                    )
                });
                let rules = Rules {
                    replace: replace_rules,
                    postformat_replace: post_format_replace_rules,
                    ..Default::default()
                };
                rules_by_country.insert(country_code.clone(), rules);

                Some((country_code, template))
            }
        })
        .collect();

    for (country_code, (parent_country_code, template)) in overrided_countries.into_iter() {
        let overrided_template = templates_by_country[&parent_country_code].clone();
        templates_by_country.insert(country_code.clone(), overrided_template);

        let mut add_component = None;
        if let Some(ac) = template["add_component"].as_str() {
            let part: Vec<_> = ac.split('=').collect();
            assert_eq!(part.len(), 2);
            let component = Component::from_str(part[0]);
            if let Ok(c) = component {
                // the only valid component that can be added is 'state'
                if c == Component::State {
                    add_component = Some(NewComponent {
                        component: c,
                        new_value: part[1].to_owned(),
                    });
                }
            }
        }

        let mut new_rules = rules_by_country
            .get(&parent_country_code)
            .cloned()
            .unwrap_or_else(Rules::default);
        new_rules.change_country_code = Some(parent_country_code.as_str().to_owned());
        new_rules.change_country = template["change_country"].as_str().map(|s| s.to_string());
        new_rules.add_component = add_component;
        rules_by_country.insert(country_code.clone(), new_rules);
    }

    let state_codes_file = include_str!("../address-formatting/conf/state_codes.yaml");
    let state_codes: HashMap<String, HashMap<String, String>> =
        serde_yaml::from_str(state_codes_file).expect("invalid state_codes.yaml file");
    let state_codes = state_codes
        .into_iter()
        .flat_map(|(country, states)| {
            states.into_iter().map(move |(state_code, state_name)| {
                (
                    (
                        CountryCode::from_str(&country).expect("invalid country code"),
                        state_name,
                    ),
                    state_code,
                )
            })
        })
        .collect();
    let county_codes_file = include_str!("../address-formatting/conf/county_codes.yaml");
    let county_codes: HashMap<String, HashMap<String, String>> =
        serde_yaml::from_str(county_codes_file).expect("invalid county_codes.yaml file");
    let county_codes = county_codes
        .into_iter()
        .flat_map(|(country, counties)| {
            counties.into_iter().map(move |(county_code, county_name)| {
                (
                    (
                        CountryCode::from_str(&country).expect("invalid country code"),
                        county_name,
                    ),
                    county_code,
                )
            })
        })
        .collect();

    let templates = Templates {
        default_template,
        fallback_template,
        templates_by_country,
        fallback_templates_by_country,
        rules_by_country,
        fallback_rules: Rules::default(),
    };
    Formatter {
        templates,
        state_codes,
        county_codes,
    }
}

pub fn read_place_builder_configuration() -> PlaceBuilder {
    let component_file = include_str!("../address-formatting/conf/components.yaml");
    let raw_components = yaml_rust::YamlLoader::load_from_str(component_file)
        .expect("impossible to read components.yaml file");
    let mut component_aliases = HashMap::<_, _>::new();

    for c in &raw_components {
        if let Some(aliases) = c["aliases"].as_vec() {
            let name = c["name"].as_str().unwrap();
            let component = Component::from_str(name)
                .unwrap_or_else(|err| panic!("Err {}: {} is not a valid component", err, name));
            for a in aliases {
                component_aliases
                    .entry(component)
                    .or_insert_with(Vec::new)
                    .push(a.as_str().unwrap().to_string());
            }
        }
    }

    PlaceBuilder { component_aliases }
}

fn build_template(yaml_value: &yaml_rust::Yaml) -> Result<Template, Error> {
    let addr_template = yaml_value
        .as_str()
        .ok_or_else(|| format_err!("no value to build template"))?;

    Ok(Template::new(addr_template))
}

fn read_replace(yaml_rules: &yaml_rust::Yaml) -> Vec<ReplaceRule> {
    yaml_rules
        .as_vec()
        .map(|v| {
            v.iter()
                .map(|r| {
                    let r = r.as_vec().expect("replace should be a list");
                    assert_eq!(r.len(), 2);

                    let first_val = r[0].as_str().expect("invalid replace rule");
                    if first_val.contains('=') {
                        // it's a replace on only one component
                        // the rules is written 'component=<string_to_replace'
                        let parts = first_val.split('=').collect::<Vec<_>>();
                        let component = Component::from_str(parts[0]).unwrap_or_else(|err| {
                            panic!(
                                "Err {}: in replace '{}' is not a valid component",
                                err, parts[0]
                            )
                        });
                        ReplaceRule::Component((
                            component,
                            Replacement {
                                regex: regex::RegexBuilder::new(parts[1])
                                    .multi_line(true)
                                    .build()
                                    .expect("invalid regex"),
                                replacement_value: r[1]
                                    .as_str()
                                    .expect("invalid replace rule")
                                    .to_owned(),
                            },
                        ))
                    } else {
                        // it's a replace for all components
                        ReplaceRule::All(Replacement {
                            regex: regex::RegexBuilder::new(first_val)
                                .multi_line(true)
                                .build()
                                .expect("invalid regex"),
                            replacement_value: r[1]
                                .as_str()
                                .expect("invalid replace rule")
                                .to_owned(),
                        })
                    }
                })
                .collect()
        })
        .unwrap_or_else(Vec::new)
}
