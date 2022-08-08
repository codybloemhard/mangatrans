use toml::Value;
use serde::Deserialize;

use std::{ fs };

#[derive(Deserialize, Debug)]
struct Chapter{
    manga: String,
    author: String,
    volume: usize,
    chapter: String,
    pic: Vec<Picture>,
}

#[derive(Deserialize, Debug)]
struct Picture{
    nr: usize,
    page: usize,
    characters: Vec<String>,
    location: Option<String>,
    text: Vec<Text>,
}

#[derive(Deserialize, Debug)]
struct Text{
    from: String,
    to: Option<String>,
    lines: Vec<String>,
    kmap: Option<Vec<[String; 2]>>,
    transl: Vec<String>,
}

fn main() {
    let contents = fs::read_to_string("example.toml").expect("oof");
    let chapter: Chapter = toml::from_str(&contents).expect("auw");
    println!("{:#?}", chapter);
}
