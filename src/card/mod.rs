use super::error::ProxygenError;

use super::maud::PreEscaped;

use super::regex::Regex;

mod database;
use self::database::DATABASE;

#[derive(Deserialize, Debug)]
pub enum Card {
    Creature {
        name: String,
        manacost: String,
        typeline: String,
        text: String,
        power: String,
        toughness: String,
    },
    Planeswalker {
        name: String,
        manacost: String,
        typeline: String,
        text: String,
        loyalty: u64,
    },
    Noncreature {
        name: String,
        manacost: String,
        typeline: String,
        text: String,
    },
    DoubleFaced {
        front: Box<Card>,
        back: Box<Card>,
    },
    Unimplemented {
        name: String,
        layout: String,
    },
}

lazy_static!{
    static ref RE: Regex = Regex::new(r"(?P<reminder>\(.+\))").unwrap();
}

fn prettify_oracle_text(text: &str) -> String {
    RE.replace_all(&text.replace("\u{2212}", "&minus;").replace("\n", "<br>"),
                   "<i>$reminder</i>")
}

fn escape_typeline_dash(text: &str) -> String {
    text.replace("\u{2014}", "&mdash;")
}

impl Card {
    pub fn from_name(name: &str) -> Result<Card, ProxygenError> {
        DATABASE.get(name)
    }

    #[allow(cyclomatic_complexity)]
    pub fn to_html(&self) -> String {
        match *self {
            Card::Creature { ref name,
                             ref manacost,
                             ref typeline,
                             ref text,
                             ref power,
                             ref toughness } => {
                let escaped_typeline = escape_typeline_dash(typeline);
                let pretty_text = prettify_oracle_text(text);
                let mut s = String::new();
                html!( s,
                    div class="card_frame" {
                        div class="card_inner" {
                            p class="name" { ^name }
                            p class="manacost" { ^manacost }
                            p class="typeline" { ^PreEscaped(escaped_typeline) }
                            p class="oracle_text" { ^PreEscaped(pretty_text)}
                            p class = "power_toughness" { ^power "/" ^toughness}
                        }
                    }
                )
                    .unwrap();
                s
            }
            Card::Planeswalker { ref name, ref manacost, ref typeline, ref text, ref loyalty } => {
                let escaped_typeline = escape_typeline_dash(typeline);
                let pretty_text = prettify_oracle_text(text);
                let mut s = String::new();
                html!( s,
                    div class="card_frame" {
                        div class="card_inner" {
                            p class="name" { ^name }
                            p class="manacost" { ^manacost }
                            p class="typeline" { ^PreEscaped(escaped_typeline) }
                            p class="oracle_text" { ^PreEscaped(pretty_text)}
                            p class = "loyalty" { ^loyalty }
                        }
                    }
                )
                    .unwrap();
                s
            }
            Card::Noncreature { ref name, ref manacost, ref typeline, ref text } => {
                let escaped_typeline = escape_typeline_dash(typeline);
                let pretty_text = prettify_oracle_text(text);
                let mut s = String::new();
                html!( s,
                    div class="card_frame" {
                        div class="card_inner" {
                            p class="name" { ^name }
                            p class="manacost" { ^manacost }
                            p class="typeline" { ^PreEscaped(escaped_typeline) }
                            p class="oracle_text" { ^PreEscaped(pretty_text)}
                        }
                    }
                )
                    .unwrap();
                s
            }
            Card::DoubleFaced { ref front, ref back } => {
                let front_html = front.to_html();
                let back_html = back.to_html();

                let mut s = String::new();
                html!(s,
                      ^PreEscaped(front_html)
                      ^PreEscaped(back_html)
                )
                    .unwrap();
                s
            }
            Card::Unimplemented { ref name, ref layout } => {
                let mut s = String::new();
                html!( s,
                    div class="card_frame" {
                        div class="card_inner" {
                            p class="name" { ^name }
                            p class="oracle_text" { "This type of card (" ^layout ") is not yet implemented. Go complain to the developer" }
                        }
                    }
                )
                    .unwrap();
                s
            }
        }
    }
}
