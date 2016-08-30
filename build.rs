extern crate ease;

use std::env;
use std::path::Path;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::io::Seek;
use std::io::SeekFrom;
use std::cmp::Ordering;
use ease::{Url, Request};

#[derive(Debug, PartialEq)]
enum VersionStatus {
    OutOfDate,
    UpToDate,
}

fn get_allcards_version_status() -> VersionStatus {
    let out_dir = env::var("OUT_DIR").unwrap();
    let allcards_path = Path::new(&out_dir).join("AllCards.json");

    if OpenOptions::new().read(true).open(&allcards_path).is_err() {
        return VersionStatus::OutOfDate;
    }

    let version_path = Path::new(&out_dir).join("version.json");
    let mut version_file =
        OpenOptions::new().create(true).read(true).write(true).open(&version_path).unwrap();
    let mut local_version_body = String::new();
    version_file.read_to_string(&mut local_version_body).unwrap();

    let version_url = Url::parse("http://mtgjson.com/json/version.json").unwrap();
    let remote_version_body = Request::new(version_url).get().unwrap().body;

    match local_version_body.cmp(&remote_version_body) {
        Ordering::Equal => VersionStatus::UpToDate,
        _ => {
            version_file.seek(SeekFrom::Start(0)).unwrap();
            write!(version_file, "{}", remote_version_body).unwrap();
            VersionStatus::OutOfDate
        }
    }
}

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let allcards_path = Path::new(&out_dir).join("AllCards.json");

    match get_allcards_version_status() {
        VersionStatus::UpToDate => {}
        VersionStatus::OutOfDate => {
            let mut allcards_file = OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(&allcards_path)
                .unwrap();

            let allcards_url = Url::parse("http://mtgjson.com/json/AllCards.json").unwrap();
            let allcards_body = Request::new(allcards_url).get().unwrap().body;

            write!(allcards_file, "{}", allcards_body).unwrap();

        }
    }
}
