use std::collections::BTreeMap;
use super::Card;
use super::super::error::ProxygenError;

use super::super::serde_json;

use super::super::sanitize_name;

use std::iter::FromIterator;

// http://mtgjson.com/json/AllCards.json.zip
const ALLCARDS_JSON: &'static str = include_str!("AllCards.json");

// Allow non snake case for automatic deserialize
#[allow(non_snake_case)]
#[derive(Deserialize, Debug, Clone)]
struct DatabaseEntry {
    layout: String,
    name: String,
    sanetype: String,
    names: Option<Vec<String>>,
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
        .map(|(key, value)| (sanitize_name(&key), value))
        .filter(|&(_, ref value)| {
            vec!["normal", "split", "flip", "double-faced", "leveler"]
                .contains(&value.layout.as_str())
        })); // û -> u, example: Lim-Dûl the Necromancer

    Database { map: good_map }
}

lazy_static!{
    pub static ref DATABASE: Database = make_database();
}

impl Database {
    fn get_entry(&self, card_name: &str) -> Result<DatabaseEntry, ProxygenError> {
        let sane_card_name = sanitize_name(card_name);
        match self.map.get(&sane_card_name) {
            Some(v) => Ok(v.clone()),
            None => Err(ProxygenError::InvalidCardName(String::from(sane_card_name))),
        }
    }

    pub fn get(&self, card_name: &str) -> Result<Card, ProxygenError> {
        let entry = try!(self.get_entry(card_name));

        self.parse_card(entry)
    }

    fn parse_card(&self, entry: DatabaseEntry) -> Result<Card, ProxygenError> {
        match entry.layout.as_str() {
            "normal" | "leveler" => {
                let types = entry.types.unwrap_or_default();
                if types.contains(&String::from("Creature")) {
                    Ok(Card::Creature {
                        name: entry.name,
                        manacost: entry.manaCost.unwrap_or_default(),
                        typeline: entry.sanetype,
                        text: entry.text.unwrap_or_default(),
                        power: entry.power.unwrap_or_default(),
                        toughness: entry.toughness.unwrap_or_default(),
                    })
                } else if types.contains(&String::from("Planeswalker")) {
                    Ok(Card::Planeswalker {
                        name: entry.name,
                        manacost: entry.manaCost.unwrap_or_default(),
                        typeline: entry.sanetype,
                        text: entry.text.unwrap_or_default(),
                        loyalty: entry.loyalty.unwrap_or_default(),
                    })
                } else {
                    Ok(Card::Noncreature {
                        name: entry.name,
                        manacost: entry.manaCost.unwrap_or_default(),
                        typeline: entry.sanetype,
                        text: entry.text.unwrap_or_default(),
                    })
                }
            }
            "double-faced" | "split" | "flip" => {
                let names = match entry.names {
                    Some(v) => {
                        if v.len() != 2 {
                            return Err(ProxygenError::MulticardHasMalformedNames(entry.name));
                        } else {
                            v
                        }
                    }
                    None => return Err(ProxygenError::MulticardHasNoNames(entry.name)),
                };

                let first_name = &names[0];
                let second_name = &names[1];

                let mut first_entry = try!(self.get_entry(first_name));
                let mut second_entry = try!(self.get_entry(second_name));;

                first_entry.layout = String::from("normal");
                second_entry.layout = String::from("normal");

                let first_card = try!(self.parse_card(first_entry));
                let second_card = try!(self.parse_card(second_entry));

                match entry.layout.as_str() {
                    "double-faced" => {
                        Ok(Card::DoubleFaced {
                            front: Box::new(first_card),
                            back: Box::new(second_card),
                        })
                    }
                    "split" => {
                        Ok(Card::Split {
                            left: Box::new(first_card),
                            right: Box::new(second_card),
                        })
                    }
                    "flip" => {
                        Ok(Card::Flip {
                            top: Box::new(first_card),
                            bottom: Box::new(second_card),
                        })
                    }
                    _ => unreachable!(),
                }
            }
            _ => {
                Ok(Card::Unimplemented {
                    name: entry.name,
                    layout: entry.layout,
                })
            }
        }
    }
}
