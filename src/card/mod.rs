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
    Split {
        left: Box<Card>,
        right: Box<Card>,
    },
    Flip {
        top: Box<Card>,
        bottom: Box<Card>,
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
    RE.replace_all(text, "<i>$reminder</i>")
        .lines()
        .map(|line| format!("<p class=\"oracle_p\">{}</p>", line))
        .collect()
}

impl Card {
    pub fn from_name(name: &str) -> Result<Card, ProxygenError> {
        DATABASE.get(name)
    }

    #[allow(cyclomatic_complexity)]
    fn inner_html(&self) -> String {
        match *self {
            Card::Creature { ref name,
                             ref manacost,
                             ref typeline,
                             ref text,
                             ref power,
                             ref toughness } => {
                let pretty_text = prettify_oracle_text(text);
                let mut s = String::new();
                html!( s,
                    p class="name" { ^name }
                    p class="manacost" { ^manacost }
                    p class="typeline" { ^typeline }
                    div class="oracle_div" { ^PreEscaped(pretty_text)}
                    p class = "power_toughness" { ^power "/" ^toughness }
                )
                    .unwrap();
                s
            }
            Card::Planeswalker { ref name, ref manacost, ref typeline, ref text, ref loyalty } => {
                let pretty_text = prettify_oracle_text(text);
                let mut s = String::new();
                html!( s,
                    p class="name" { ^name }
                    p class="manacost" { ^manacost }
                    p class="typeline" { ^typeline }
                    div class="oracle_div" { ^PreEscaped(pretty_text)}
                    p class = "loyalty" { ^loyalty }
                )
                    .unwrap();
                s
            }
            Card::Noncreature { ref name, ref manacost, ref typeline, ref text } => {
                let pretty_text = prettify_oracle_text(text);
                let mut s = String::new();
                html!( s,
                    p class="name" { ^name }
                    p class="manacost" { ^manacost }
                    p class="typeline" { ^typeline }
                    div class="oracle_div" { ^PreEscaped(pretty_text)}
                )
                    .unwrap();
                s
            }
            Card::Unimplemented { ref name, ref layout } => {
                let mut s = String::new();
                html!( s,
                    p class="name" { ^name }
                    div class="oracle_div" { "This type of card (" ^layout ") is not yet implemented. Go complain to the developer" }
                )
                    .unwrap();
                s
            }
            ref err => panic!("This shouldn't happen. {:?}", err),
        }
    }

    pub fn to_html(&self) -> String {
        match *self {
            Card::DoubleFaced { ref front, ref back } => {
                let front_html = front.inner_html();
                let back_html = back.inner_html();

                let mut s = String::new();
                html!(s,
                      div class="card_frame" {
                          div class="card_inner" {
                            ^PreEscaped(front_html)
                          }
                      }
                    div class="card_frame" {
                        div class="card_inner" {
                          ^PreEscaped(back_html)
                        }
                    }
                )
                    .unwrap();
                s
            }
            Card::Split { ref left, ref right } => {
                let left_html = left.inner_html();
                let right_html = right.inner_html();

                let mut s = String::new();
                html!(s,
                    div class="card_frame" {
                        div class="card_inner" {
                            div class="split_left" {
                                ^PreEscaped(left_html)
                            }
                            div class="split_bottom" {
                                ^PreEscaped(right_html)
                            }
                        }
                    }
                )
                    .unwrap();
                s
            }
            Card::Flip { ref top, ref bottom } => {
                let top_html = top.inner_html();
                let bottom_html = bottom.inner_html();

                let mut s = String::new();
                html!(s,
                    div class="card_frame" {
                        div class="card_inner" {
                            div class="flip_top" {
                                ^PreEscaped(top_html)
                            }
                            div class="flip_bottom" {
                                ^PreEscaped(bottom_html)
                            }
                        }
                    }
                )
                    .unwrap();
                s
            }
            _ => {
                let mut s = String::new();
                html!( s,
                    div class="card_frame" {
                        div class="card_inner" {
                            ^PreEscaped(self.inner_html())
                        }
                    }
                )
                    .unwrap();
                s
            }
        }
    }
}
