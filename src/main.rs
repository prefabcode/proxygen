#![feature(plugin)]
#![plugin(clippy)]

#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

#![feature(plugin)]
#![plugin(maud_macros)]

extern crate serde;
extern crate serde_json;

extern crate maud;
use maud::PreEscaped;

#[macro_use]
extern crate nickel;
use nickel::{Nickel, HttpRouter, FormBody};
use nickel::status::StatusCode;

extern crate regex;
use regex::Regex;

#[macro_use]
extern crate lazy_static;

mod card;
use card::Card;
mod error;
use error::ProxygenError;

const PROXYGEN_CSS: &'static str = include_str!("proxygen.css");
const RESULTS_CSS: &'static str = include_str!("results.css");
const MAX_CARDS: u64 = 1000;

lazy_static!{
    static ref BASE_RE: Regex = Regex::new(r"(\d+)?x?\s*(\D*?)\s*$").unwrap();
    static ref SPLIT_RE: Regex = Regex::new(r"(.+?)\s*/+\s*.+").unwrap();
}

fn parse_decklist(decklist: &str) -> Result<Vec<(u64, Card)>, ProxygenError> {
    let mut count = 0;
    let mut out = Vec::new();
    for entry in decklist.lines() {
        let trimmed = entry.trim();
        if !trimmed.is_empty() {
            let (n, c) = match BASE_RE.captures(trimmed) {
                Some(captures) => {
                    let amount: u64 = match captures.at(1) {
                        Some(v) => v.parse().unwrap(),
                        None => 1,
                    };

                    count += amount;
                    if count > MAX_CARDS {
                        return Err(ProxygenError::TooManyCards);
                    }

                    let card_name = captures.at(2).unwrap();

                    let sane_card_name = match SPLIT_RE.captures(card_name) {
                        Some(split_captures) => split_captures.at(1).unwrap(),
                        None => card_name,
                    };

                    let card = match card::Card::from_name(sane_card_name) {
                        Ok(v) => v,
                        Err(e) => {
                            return Err(e);
                        }
                    };

                    (amount, card)
                }
                None => return Err(ProxygenError::DecklistParseError(String::from(trimmed))),
            };
            out.push((n, c));
        };
    }
    Ok(out)
}

fn main() {
    println!("Building database..");
    Card::from_name("Island").unwrap_or_else(|e| panic!("Error building database: {:?}", e));

    let mut server = Nickel::new();

    server.get("/proxygen",
               middleware!(|_req, res| {
        let mut doc = String::new();
        html!(doc, html {
            head {
                meta charset="UTF-8"
                title { "Proxygen" }
                link href="https://fonts.googleapis.com/css?family=Inconsolata|Open+Sans"
                     rel="stylesheet"
                style {
                    ^PreEscaped(PROXYGEN_CSS)
                }
            }
            body {
                div id="surround" {
                    div id="content" {
                        h1 { "Simple Proxy Generator" }
                        form method="post" {
                            textarea name="decklist" class="decklist" {
                              "1 Snapcaster Mage\r\n"
                              "2x Ponder\r\n"
                              "Dance of the Dead\r\n"
                              "Stomping Ground\r\n"
                              "Jace, the Mind Sculptor\r\n"
                              "Delver of Secrets\r\n"
                              "Ice\r\n"
                              "Fire // Ice\r\n"
                              "Akki Lavarunner\r\n"
                              "Echo Mage\r\n"
                              "Æthersnipe\r\n"
                              "Aethersnipe\r\n"
                              "Anafenza, Kin-Tree Spirit\r\n"
                              "Anafenza Kin Tree Spirit\r\n"
                            }
                            input type="submit" /
                        }
                        p {
                            "Please report any errors "
                            "to the issue tracker on my "
                            a href="https://github.com/Dryvnt/proxygen" {
                                "Github project page"
                            }
                            "."
                        }
                    }
                }
            }
        }).unwrap();
        return res.send(doc)
    }));

    server.post("/proxygen",
                middleware!(|req, mut res| {
        let form_body = try_with!(res, req.form_body());
        let decklist = String::from(match form_body.get("decklist") {
            Some(v) => v,
            None => {
                *res.status_mut() = StatusCode::BadRequest;
                return res.send("POST request form did not contain decklist")
            }
        });

        let parsed = match parse_decklist(&decklist) {
            Ok(v) => {
                println!("{:?}", decklist);
                v
            },
            Err(e) => {
                println!("{:?}\n\t{:?}", decklist, e);
                match e {
                    ProxygenError::TooManyCards => {
                        *res.status_mut() = StatusCode::BadRequest;
                        return res.send(format!("Too many proxies requested.
                            Request at most {} proxies at a time", MAX_CARDS))
                    }
                    ProxygenError::InvalidCardName(s) => {
                        *res.status_mut() = StatusCode::BadRequest;
                        return res.send(format!("Invalid card name: {:?}", s));
                    },
                    ProxygenError::DecklistParseError(s) => {
                        *res.status_mut() = StatusCode::BadRequest;
                        return res.send(format!("Error parsing decklist at line: {:?}", s));
                    },
                    ProxygenError::MulticardHasMalformedNames(s) => {
                        *res.status_mut() = StatusCode::InternalServerError;
                        return res.send(format!("A split/flip/transform/meld card
                            has less than two forms: {:?}", s))
                    }
                    e => {
                        *res.status_mut() = StatusCode::InternalServerError;
                        return res.send(format!("An error happened interally that wasn't
                            handled properly. Tell the developer '{:?}'", e));
                    }
                }
            }

        };

        let mut div_chain = String::new();

        for pair in parsed {
            let (n, card) = pair;
            for _ in 0..n {
                div_chain.push_str(&card.to_html());
            }
        }

        let mut doc = String::new();
        html!(doc, html {
            head {
                meta charset="UTF-8"
                title { "Proxygen" }
                link href="https://fonts.googleapis.com/css?family=Open+Sans" rel="stylesheet"
                style {
                    ^PreEscaped(RESULTS_CSS)
                }
            }
            body {
                ^PreEscaped(div_chain)
            }
        }).unwrap();
        return res.send(doc)
    }));

    server.listen("127.0.0.1:6767");
}
