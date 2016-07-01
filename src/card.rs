use super::error::ProxygenError;

use super::ease::{Url, Request};

use super::maud::PreEscaped;

use super::serde_json;

#[derive(Deserialize, Debug)]
pub struct Card {
    // There are other fields we omit for simplicity
    name: String,
    cost: String,
    types: Vec<String>,
    subtypes: Option<Vec<String>>,
    text: String,
    power: Option<String>,
    toughness: Option<String>,
}

fn prettify_types(types: &[String]) -> String {
    let mut collected = String::new();
    for t in types {
        let (first, rest) = t.split_at(1);
        let mut first = first.to_uppercase();
        first.push_str(rest);
        if !collected.is_empty() {
            collected.push(' ');
        }
        collected.push_str(&first);
    }
    collected
}

impl Card {
    pub fn from_name(name: &str) -> Result<Card, ProxygenError> {
        let sane_name = String::from(name).to_lowercase().replace(" ", "-");
        let url = Url::parse(&(String::from("https://api.deckbrew.com/mtg/cards/") + &sane_name))
            .unwrap();

        match Request::new(url).get() {
            Err(_) => Err(ProxygenError::InvalidCardName(String::from(name))),
            Ok(res) => {
                match serde_json::from_str::<Card>(&res.body) {
                    Err(e) => Err(ProxygenError::from(e)),
                    Ok(card) => Ok(card),
                }
            }
        }
    }

    pub fn to_html(&self) -> String {
        let mut s = String::new();

        let sane_cost = self.cost.replace("{", "").replace("}", "");

        // TODO: functionize this, somehow?
        let collected_types = prettify_types(&self.types);
        let sane_list = vec![];
        let subtypes_list = match self.subtypes {
            Some(ref v) => v,
            None => &sane_list,
        };
        let collected_subtypes = prettify_types(subtypes_list);

        let sane_subtypes = if collected_subtypes.is_empty() {
            collected_subtypes
        } else {
            String::from(" - ") + &collected_subtypes
        };

        let sane_text = self.text.replace("\n", "<br>");

        let sane_pt = match self.power {
            Some(ref pow) => pow.clone() + "/" + &self.toughness.clone().unwrap(),
            None => String::new(),
        };

        html!(s, {
            p { b {^self.name} " (" ^sane_cost ")" }
            p { ^collected_types ^sane_subtypes }
            p { ^PreEscaped(sane_text) }
            p style="text-align: right;" { ^sane_pt }
        })
            .unwrap();

        s
    }
}
