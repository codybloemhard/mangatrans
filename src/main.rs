use serde::Deserialize;

use std::fs;
use std::fmt::Write;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
struct Chapter{
    manga: String,
    author: String,
    volume: usize,
    chapter: usize,
    title: String,
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
    let chapter = match toml::from_str::<Chapter>(&contents){
        Ok(chapter) => chapter,
        Err(error) => panic!("{} (error position is an estimation!)", error),
    };
    let md = write_transcription(chapter);
    println!("{}", md);
}

fn write_transcription(chapter: Chapter) -> String{
    let mut md = String::new();
    writeln!(md, "{}{}", header(1), &chapter.title);
    writeln!(md, "Manga: {}", chapter.manga);
    writeln!(md, "Author: {}", chapter.author);
    writeln!(md, "Volume: {}", chapter.volume);
    writeln!(md, "Chapter: {}", chapter.chapter);

    let mut last_page = 0;
    for picture in chapter.pic{
        if picture.page > last_page{
            writeln!(md, "{}Page: {}", header(5), picture.page);
            last_page = picture.page;
        }
        writeln!(md, "{}picture {}", bullet(1), picture.nr);

        fn write_text(md: &mut String, ident: usize, text: &Text){
            fn write_lines(md: &mut String, lines: &[String]) {
                for line in lines{
                    write!(md, "{} <br/> ", line.replace(" ", ""));
                }
                for _ in 0..7 { md.pop(); }
            }
            // transcription
            write!(md, "{}", bullet(ident + 1));
            write_lines(md, &text.lines);
            writeln!(md);
            // kanji replacement
            let mut replacements = Vec::new();
            let mut map = HashMap::new();
            if let Some(kmap) = &text.kmap{
                for [x, y] in kmap{
                    map.insert(x, y);
                }
            }
            for line in &text.lines{
                let mut replaced = String::new();
                for c in line.replace(" ", "").chars(){
                    let s = c.to_string();
                    if let Some(replacement) = map.get(&s){
                        replaced.push_str(replacement);
                    } else {
                        replaced.push_str(&s);
                    }
                }
                replacements.push(replaced);
            }
            write!(md, "{}", bullet(ident + 1));
            write_lines(md, &replacements);
            writeln!(md);
        }

        if picture.text.len() == 1{
            write_text(&mut md, 2, &picture.text[0]);
        } else {
            for (n, text) in picture.text.iter().enumerate(){
                writeln!(md, "{}text {}", bullet(2), n);
                write_text(&mut md, 3, text);
            }
        }
    }
    md
}

fn header(rank: usize) -> String{
    let mut temp = "#".repeat(rank).to_string();
    temp.push_str(" ");
    temp
}

fn bullet(ident: usize) -> String{
    let mut temp = "  ".repeat(ident).to_string();
    temp.push_str("- ");
    temp
}
