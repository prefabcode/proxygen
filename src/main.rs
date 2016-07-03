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
use nickel::{Nickel, FormBody, MediaType};
use nickel::status::StatusCode;

extern crate regex;
use regex::Regex;

#[macro_use]
extern crate lazy_static;

mod card;
use card::Card;
mod error;
use error::ProxygenError;


const PROXYGEN_HTML: &'static str = include_str!("proxygen.html");
const PROXYGEN_CSS: &'static str = include_str!("proxygen.css");
const RESULTS_CSS: &'static str = include_str!("results.css");

lazy_static!{
    static ref RE: Regex = Regex::new(r"^\s*(\d+)?x?\s*(\D*?)\s*$").unwrap();
}

fn parse_decklist(decklist: &str) -> Result<Vec<(u32, Card)>, ProxygenError> {
    decklist.lines()
        .map(|entry| entry.trim())
        .filter(|entry| !entry.is_empty())
        .map(|card_entry| {
            match RE.captures(card_entry) {
                Some(captures) => {
                    let amount: u32 = match captures.at(1) {
                        Some(v) => v.parse().unwrap(),
                        None => 1,
                    };

                    let card_name = captures.at(2).unwrap();

                    let card = match card::Card::from_name(card_name) {
                        Ok(v) => v,
                        Err(e) => return Err(e),
                    };

                    Ok((amount, card))
                }
                None => Err(ProxygenError::DecklistParseError(String::from(card_entry))),
            }

        })
        .collect()
}

fn main() {
    let mut server = Nickel::new();

    // Build database
    println!("Building database..");
    Card::from_name("Island").unwrap_or_else(|e| panic!("Error building database: {:?}", e));

    server.utilize(router!{
        post "/proxygen" => |req, mut res| {
            let form_body = try_with!(res, req.form_body());
            println!("{:?}", form_body);
            let decklist = String::from(form_body.get("decklist").unwrap());

            let parsed = match parse_decklist(&decklist) {
                Ok(v) => v,
                Err(ProxygenError::InvalidCardName(s)) => {
                    *res.status_mut() = StatusCode::BadRequest;
                    return res.send(format!("Invalid card name: {:?}", s));
                },
                Err(ProxygenError::DecklistParseError(s)) => {
                    *res.status_mut() = StatusCode::BadRequest;
                    return res.send(format!("Error parsing decklist at line: {:?}", s));
                },
                Err(ProxygenError::MulticardHasMalformedNames(s)) => {
                    *res.status_mut() = StatusCode::InternalServerError;
                    return res.send(format!("A split/flip/transform has more than 2 different forms. Are you using unhinged/unglued cards? Card: {:?}", s))
                }
                Err(e) => {
                    *res.status_mut() = StatusCode::InternalServerError;
                    return res.send(format!("An error happened interally that wasn't handled properly. Tell the developer '{:?}'"), e);
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
                    style {
                        ^PreEscaped(RESULTS_CSS)
                    }
                }
                body {
                    ^PreEscaped(div_chain)
                }
            }).unwrap();
            return res.send(doc)
        }
    });

    // Static files
    server.utilize(router! {
        get "/proxygen.css" => |_req, mut res| {
            res.set(MediaType::Css);

            return res.send(PROXYGEN_CSS)
        }
        get "/proxygen" => |_req, res| {
            return res.send(PROXYGEN_HTML)
        }
    });

    server.listen("127.0.0.1:6767");
}
