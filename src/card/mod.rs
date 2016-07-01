use super::error::ProxygenError;

use super::maud::PreEscaped;

use super::regex::Regex;

mod database;
use self::database::DATABASE;

#[derive(Deserialize, Debug)]
pub struct Card {
    // There are other fields we omit for simplicity
    name: String,
    cost: String,
    typeline: String,
    text: String,
    power_toughness: Option<(String, String)>,
}

lazy_static!{
    static ref RE: Regex = Regex::new(r"(?P<reminder>\(.+\))").unwrap();
}

impl Card {
    pub fn from_name(name: &str) -> Result<Card, ProxygenError> {
        DATABASE.get(name)
    }

    // TODO: circumvent this? see sane_pt
    #[allow(needless_borrow)]
    pub fn to_html(&self) -> String {
        let mut s = String::new();

        let typeline = self.typeline.replace("\u{2014}", "&mdash;");

        let sane_text = self.text.replace("\n", "<br>");
        let pretty_text = RE.replace_all(&sane_text, "<i>$reminder</i>");

        let sane_pt = match self.power_toughness {
            Some((ref pow, ref tou)) => pow.clone() + "/" + &tou,
            None => String::new(),
        };

        html!(s, {
            p { b {^self.name} " " ^self.cost }
            p { ^PreEscaped(typeline) }
            p { ^PreEscaped(pretty_text) }
            p style="text-align: right;" { ^sane_pt }
        })
            .unwrap();

        s
    }
}
