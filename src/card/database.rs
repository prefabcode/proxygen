use std::collections::BTreeMap;
use super::Card;
use super::super::error::ProxygenError;

use super::super::serde_json;

use std::iter::FromIterator;

const ALLCARDS_JSON: &'static str = include_str!("AllCards.json"); //http://mtgjson.com/json/AllCards.json.zip

#[derive(Deserialize, Debug, Clone)]
struct database_entry {
    name: String,
    manaCost: Option<String>,
    supertypes: Option<Vec<String>>,
    types: Option<Vec<String>>,
    subtypes: Option<Vec<String>>,
    text: Option<String>,
    power: Option<String>,
    toughness: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct database {
    map: BTreeMap<String, database_entry>,
}

fn make_database() -> database {
    let mut bad_map: BTreeMap<String, database_entry> = serde_json::from_str(ALLCARDS_JSON)
        .unwrap();

    let mut good_map = BTreeMap::from_iter(bad_map.iter().map(|(bad_key, value)| {
        let mut good_key = bad_key.clone();
        let good_value = value.clone();

        good_key = good_key.replace("\u{fb}", "u");

        (good_key, good_value)

    }));
    database { map: good_map }
}

lazy_static!{
    pub static ref DATABASE: database = make_database();
}

impl database {
    pub fn get(&self, card_name: &str) -> Result<Card, ProxygenError> {
        let entry = match self.map.get(card_name) {
            Some(v) => v,
            None => return Err(ProxygenError::InvalidCardName(String::from(card_name))),
        };

        // TODO: Get rid of these clones, jesus christ

        Ok(Card {
            name: entry.name.clone(),
            cost: entry.manaCost.clone().unwrap_or(String::new()),
            typeline: {
                let mut typeline = String::new();
                for t in entry.supertypes.clone().unwrap_or(vec![]) {
                    typeline.push_str(&t);
                    typeline.push_str(" ");
                }
                for t in entry.types.clone().unwrap_or(vec![]) {
                    typeline.push_str(&t);
                    typeline.push_str(" ");
                }
                if entry.subtypes.is_some() {
                    typeline.push_str(" \u{2014}");
                    for t in entry.subtypes.clone().unwrap_or(vec![]) {
                        typeline.push_str(" ");
                        typeline.push_str(&t);
                    }
                }
                typeline
            },
            text: entry.text.clone().unwrap_or(String::new()),
            power_toughness: {
                match entry.power.clone() {
                    Some(pow) => {
                        let tou = entry.toughness.clone().unwrap();
                        Some((pow, tou))
                    }
                    None => None,
                }
            },
        })
    }
}
