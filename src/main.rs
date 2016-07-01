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

use nickel::Nickel;

extern crate regex;
use regex::Regex;
use regex::Nickel::{Request, Response};

#[macro_use]
extern crate lazy_static;

mod card;
mod error;

use std::io::Read;

const PROXYGEN_HTML: &'static str = include_str!("proxygen.html");
const PROXYGEN_CSS: &'static str = include_str!("proxygen.css");

fn handle_post(mut req: Request, mut res: Response) {
    let mut buffer = String::new();
    req.read_to_string(&mut buffer).unwrap();

    lazy_static! {
        static ref RE: Regex = Regex::new("decklist=(.*)").unwrap();
    }


    println!("{:?}", buffer);
    res.send(b"<p>lol</p>").unwrap();
}

fn main() {
    server.utilize(router! {
        get "proxygen.html" => |_req, _res| {
            PROXYGEN_HTML
        }
        get "proxygen.css" => |_req, _res| {
            PROXYGEN_CSS
        }
        post "proxygen.html" => |req, res| {
            handle_post(req, res)
        }
    });
    Server::http("127.0.0.1:6767").unwrap().handle(hello).unwrap();
}
