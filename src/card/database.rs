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
    sanetype: String,
    manaCost: Option<String>,
    supertypes: Option<Vec<String>>,
    types: Option<Vec<String>>,
    subtypes: Option<Vec<String>>,
    text: Option<String>,
    power: Option<String>,
    toughness: Option<String>,
    loyalty: Option<u64>,
}

#[derive(Debug)]
pub struct Database {
    map: BTreeMap<String, DatabaseEntry>,
}

fn make_database() -> Database {
    let sane_allcards_json = String::from(ALLCARDS_JSON).replace("\"type\":", "\"sanetype\":");
    let bad_map: BTreeMap<String, DatabaseEntry> = serde_json::from_str(&sane_allcards_json)
        .unwrap();

    let good_map: BTreeMap<String, DatabaseEntry> = BTreeMap::from_iter(bad_map.iter()
        .map(|(key, value)| (key.clone(), value.clone()))
        .filter(|&(_, ref value)| {
            vec!["normal", "split", "flip", "double-faced", "leveler"]
                .contains(&value.layout.as_str())
        })
        .map(|(key, value)| (key.replace("\u{fb}", "u"), value))); // û -> u, example: Lim-Dûl the Necromancer

    Database { map: good_map }
}

lazy_static!{
    pub static ref DATABASE: Database = make_database();
}

fn parse_card(entry: DatabaseEntry) -> Card {
    match entry.layout.as_str() {
        "normal" => {
            let types = entry.types.unwrap_or_default();
            if types.contains(&String::from("Creature")) {
                Card::Creature {
                    name: entry.name,
                    manacost: entry.manaCost.unwrap_or_default(),
                    typeline: entry.sanetype,
                    text: entry.text.unwrap_or_default(),
                    power: entry.power.unwrap_or_default(),
                    toughness: entry.toughness.unwrap_or_default(),
                }
            } else if types.contains(&String::from("Planeswalker")) {
                Card::Planeswalker {
                    name: entry.name,
                    manacost: entry.manaCost.unwrap_or_default(),
                    typeline: entry.sanetype,
                    text: entry.text.unwrap_or_default(),
                    loyalty: entry.loyalty.unwrap_or_default(),
                }
            } else {
                Card::Noncreature {
                    name: entry.name,
                    manacost: entry.manaCost.unwrap_or_default(),
                    typeline: entry.sanetype,
                    text: entry.text.unwrap_or_default(),
                }

            }
        }
        _ => {
            Card::Unimplemented {
                name: entry.name,
                layout: entry.layout,
            }
        }
    }
}

impl Database {
    fn get_entry(&self, card_name: &str) -> Result<DatabaseEntry, ProxygenError> {
        match self.map.get(card_name) {
            Some(v) => Ok(v.clone()),
            None => Err(ProxygenError::InvalidCardName(String::from(card_name))),
        }
    }

    pub fn get(&self, card_name: &str) -> Result<Card, ProxygenError> {
        let entry = try!(self.get_entry(card_name));

        Ok(parse_card(entry))
    }
}
