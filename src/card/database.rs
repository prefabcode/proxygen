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


impl Database {
    fn get_entry(&self, card_name: &str) -> Result<DatabaseEntry, ProxygenError> {
        match self.map.get(card_name) {
            Some(v) => Ok(v.clone()),
            None => Err(ProxygenError::InvalidCardName(String::from(card_name))),
        }
    }

    pub fn get(&self, card_name: &str) -> Result<Card, ProxygenError> {
        let entry = try!(self.get_entry(card_name));

        self.parse_card(entry)
    }

    fn parse_card(&self, entry: DatabaseEntry) -> Result<Card, ProxygenError> {
        match entry.layout.as_str() {
            "normal" => {
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
            "double-faced" => {
                let mut names = entry.names.unwrap_or_default();
                let back_name = names.pop().unwrap_or_default();
                let front_name = names.pop().unwrap_or_default();

                let mut front_entry = try!(self.get_entry(&front_name));
                let mut back_entry = try!(self.get_entry(&back_name));;

                front_entry.layout = String::from("normal");
                back_entry.layout = String::from("normal");

                let front_card = try!(self.parse_card(front_entry));
                let back_card = try!(self.parse_card(back_entry));

                Ok(Card::DoubleFaced {
                    front: Box::new(front_card),
                    back: Box::new(back_card),
                })
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
