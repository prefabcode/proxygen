use std::collections::BTreeMap;
use super::Card;
use super::super::error::ProxygenError;

use super::super::serde_json;

use std::iter::FromIterator;

const ALLCARDS_JSON: &'static str = include_str!("AllCards.json"); //http://mtgjson.com/json/AllCards.json.zip

// Allow non snake case for automatic deserialize
#[allow(non_snake_case)]
#[derive(Deserialize, Debug, Clone)]
struct DatabaseEntry {
    layout: String,
    name: String,
    sanetype: Option<String>,
    manaCost: Option<String>,
    supertypes: Option<Vec<String>>,
    types: Option<Vec<String>>,
    subtypes: Option<Vec<String>>,
    text: Option<String>,
    power: Option<String>,
    toughness: Option<String>,
    loyalty: Option<u64>,
}

#[derive(Deserialize, Debug)]
pub struct Database {
    map: BTreeMap<String, DatabaseEntry>,
}

fn make_database() -> Database {
    let sane_allcards_json = String::from(ALLCARDS_JSON).replace("\"type\":", "\"sanetype\":");
    let bad_map: BTreeMap<String, DatabaseEntry> = serde_json::from_str(&sane_allcards_json)
        .unwrap();

    let good_map = BTreeMap::from_iter(bad_map.iter().map(|(bad_key, value)| {
        let mut good_key = bad_key.clone();
        let good_value = value.clone();

        good_key = good_key.replace("\u{fb}", "u");

        (good_key, good_value)

    }));
    Database { map: good_map }
}

lazy_static!{
    pub static ref DATABASE: Database = make_database();
}

impl Database {
    pub fn get(&self, card_name: &str) -> Result<Card, ProxygenError> {
        let entry = match self.map.get(card_name) {
            Some(v) => v.clone(),
            None => return Err(ProxygenError::InvalidCardName(String::from(card_name))),
        };


        // TODO: Get rid of these clones, jesus christ

        match entry.layout.as_str() {
            "normal" => {
                let types = entry.types.unwrap_or_default();
                if types.contains(&String::from("Creature")) {
                    Ok(Card::Creature {
                        name: entry.name,
                        manacost: entry.manaCost.unwrap_or_default(),
                        typeline: entry.sanetype.unwrap_or_default(),
                        text: entry.text.unwrap_or_default(),
                        power: entry.power.unwrap_or_default(),
                        toughness: entry.toughness.unwrap_or_default(),
                    })
                } else if types.contains(&String::from("Planeswalker")) {
                    Ok(Card::Planeswalker {
                        name: entry.name,
                        manacost: entry.manaCost.unwrap_or_default(),
                        typeline: entry.sanetype.unwrap_or_default(),
                        text: entry.text.unwrap_or_default(),
                        loyalty: entry.loyalty.unwrap_or_default(),
                    })
                } else {
                    Ok(Card::Noncreature {
                        name: entry.name,
                        manacost: entry.manaCost.unwrap_or_default(),
                        typeline: entry.sanetype.unwrap_or_default(),
                        text: entry.text.unwrap_or_default(),
                    })

                }
            }
            _ => {
                Ok((Card::Unimplemented {
                    name: entry.name,
                    layout: entry.layout,
                }))
            }
        }

    }
}
