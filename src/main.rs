#![feature(plugin)]
#![plugin(clippy)]

#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

#![feature(plugin)]
#![plugin(maud_macros)]

extern crate ease;

extern crate serde;
extern crate serde_json;

extern crate maud;
use maud::PreEscaped;

#[macro_use]
extern crate nickel;
use nickel::{Nickel, NickelError, FormBody, MediaType};
use nickel::status::StatusCode;

extern crate regex;
use regex::Regex;

#[macro_use]
extern crate lazy_static;

mod card;
use card::Card;
mod error;
use error::ProxygenError;

use std::io::Read;

const PROXYGEN_HTML: &'static str = include_str!("proxygen.html");
const PROXYGEN_CSS: &'static str = include_str!("proxygen.css");

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

                    println!("{}x {:?}", amount, card);

                    Ok((amount, card))
                }
                None => return Err(ProxygenError::DecklistParseError(String::from(card_entry))),
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
            let decklist = String::from(form_body.get("decklist").unwrap());

            let parsed = match parse_decklist(&decklist) {
                Ok(v) => v,
                Err(ProxygenError::InvalidCardName(s)) => {
                    *res.status_mut() = StatusCode::BadRequest;
                    return res.send(format!("Invalid card name: {:?}", s));
                },
                _ => {
                    panic!("unimplemented error code")
                }
            };

            let mut div_chain = String::new();

            for pair in parsed {
                let (n, card) = pair;
                for i in 0..n {
                    html!(div_chain, {
                        div style="border: 0.5mm solid black; float: left; width: 60mm; height: 85mm" {
                            div style="padding: 2mm" {
                                ^PreEscaped(card.to_html())
                            }
                        }
                    }).unwrap();
                }
            }

            //decklist.replace("")
            println!("{:?}", form_body);
            return res.send(div_chain)
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
