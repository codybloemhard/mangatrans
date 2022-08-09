use serde::Deserialize;
use clap::Parser;

use std::fs;
use std::fmt::Write;
use std::collections::HashMap;
use std::io::Write as _;

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

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args{
    #[clap(short, long, default_value = "file")]
    outputmode: String,
    #[clap(short, long)]
    files: Vec<String>,
}

fn main() {
    let args = Args::parse();
    for file in args.files{
        let contents = match fs::read_to_string(&file){
            Ok(contents) => contents,
            Err(error) => {
                println!("Could not read file: \"{}\".\n\tError: {}", file, error);
                continue;
            }
        };
        let chapter = match toml::from_str::<Chapter>(&contents){
            Ok(chapter) => chapter,
            Err(error) => panic!("{} (error position is an estimation!)", error),
        };
        let md = write_transcription(chapter);
        if args.outputmode == "file"{
            let outname = if file.contains(".toml"){
                file.replace(".toml", ".md")
            } else {
                let mut string = file.clone();
                string.push_str(".md");
                string
            };
            let mut outfile = match fs::File::create(&outname){
                Ok(outfile) => outfile,
                Err(error) => {
                    println!("Could not create file: \"{}\".\n\tError: {}", outname, error);
                    continue;
                }
            };
            if let Err(error) = write!(outfile, "{}", md){
                println!("Could not write to file: \"{}\".\n\tError: {}", outname, error);
            }
        } else {
            println!("{}", md);
        }
    }
}

fn write_transcription(chapter: Chapter) -> String{
    let mut md = String::new();
    let _ = writeln!(md, "{}{}", header(1), &chapter.title);
    let _ = writeln!(md, "Manga: {}", chapter.manga);
    let _ = writeln!(md, "Author: {}", chapter.author);
    let _ = writeln!(md, "Volume: {}", chapter.volume);
    let _ = writeln!(md, "Chapter: {}", chapter.chapter);

    let mut last_page = 0;
    for picture in chapter.pic{
        if picture.page > last_page{
            let _ = writeln!(md, "{}Page: {}", header(5), picture.page);
            last_page = picture.page;
        }
        let _ = writeln!(md, "{}picture {}", bullet(1), picture.nr);

        fn write_text(md: &mut String, ident: usize, text: &Text){
            fn write_lines(md: &mut String, lines: &[String], rep: &str) {
                for line in lines{
                    let _ = write!(md, "{} <br/> ", line.replace(" ", rep));
                }
                for _ in 0..7 { md.pop(); }
            }
            // transcription
            let _ = write!(md, "{}", bullet(ident + 1));
            write_lines(md, &text.lines, "");
            let _ = writeln!(md);
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
                for c in line.chars(){
                    let s = c.to_string();
                    if let Some(replacement) = map.get(&s){
                        replaced.push_str(replacement);
                    } else {
                        replaced.push_str(&s);
                    }
                }
                replacements.push(replaced);
            }
            let _ = write!(md, "{}", bullet(ident + 1));
            write_lines(md, &replacements, "");
            let _ = writeln!(md);
            // romanize
            let mut romanizeds = Vec::new();
            for rep in &replacements{
                romanizeds.push(romanize(rep));
            }
            let _ = write!(md, "{}", bullet(ident + 1));
            write_lines(md, &romanizeds, " ");
            let _ = writeln!(md);
            // translation
            let _ = write!(md, "{}", bullet(ident + 1));
            write_lines(md, &text.transl, " ");
            let _ = writeln!(md);
        }

        if picture.text.len() == 1{
            write_text(&mut md, 2, &picture.text[0]);
        } else {
            for (n, text) in picture.text.iter().enumerate(){
                let _ = writeln!(md, "{}text {}", bullet(2), n);
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

fn romanize(string: &str) -> String{
    let mut res = String::new();
    let chars: Vec<char> = string.chars().collect();
    if chars.is_empty() { return res; }
    let lm1 = chars.len() - 1;
    let mut i = 0;
    let mut prev = chars[0];
    let mut tsu = false;

    fn push(res: &mut String, roman: &str, tsu: &mut bool, prev: &mut char){
        if *tsu{
            let next = roman.chars().next();
            if let Some(next) = next{
                res.push_str(&next.to_string());
            }
            *tsu = false;
        }
        res.push_str(roman);
        if let Some(last) = roman.chars().last(){ *prev = last; }
    }

    while i < lm1{
        let a = chars[i];
        let b = chars[i + 1];
        let comb = format!("{}{}", a, b);
        match Hepburn::from(&comb){
            Hepburn::Roman(roman) => {
                push(&mut res, &roman, &mut tsu, &mut prev);
                i += 2;
                continue;
            },
            _ => {},
        }
        let a = a.to_string();
        match Hepburn::from(&a){
            Hepburn::Roman(roman) => push(&mut res, &roman, &mut tsu, &mut prev),
            Hepburn::SmallTsu => tsu = true,
            Hepburn::Enlongate => res.push(prev),
            Hepburn::Fail => res.push_str(&a),
        }
        i += 1;
    }
    if i == lm1{
        let last = chars[lm1].to_string();
        if let Hepburn::Roman(roman) = Hepburn::from(&last){
            push(&mut res, &roman, &mut tsu, &mut prev);
        } else {
            res.push_str(&last);
        }
    }
    res
}

enum Hepburn{ Roman(String), SmallTsu, Enlongate, Fail }

impl Hepburn{
    fn from(string: &str) -> Self{
        let temp = match string{
            "あ" => "a", "ア" => "a", "い" => "i", "イ" => "i", "う" => "u", "ウ" => "u",
            "え" => "e", "エ" => "e", "お" => "o", "オ" => "o",
            "か" => "ka", "カ" => "ka", "き" => "ki", "キ" => "ki", "く" => "ku", "ク" => "ku",
            "け" => "ke", "ケ" => "ke", "こ" => "ko", "コ" => "ko",
            "さ" => "sa", "サ" => "sa", "し" => "shi", "シ" => "shi", "す" => "su", "ス" => "su",
            "せ" => "se", "セ" => "se", "そ" => "so", "ソ" => "so",
            "た" => "ta", "タ" => "ta", "ち" => "chi", "チ" => "chi", "つ" => "tsu", "ツ" => "tsu",
            "て" => "te", "テ" => "te", "と" => "to", "ト" => "to", "な" => "na", "ナ" => "na",
            "に" => "ni", "ニ" => "ni", "ぬ" => "nu", "ヌ" => "nu", "ね" => "ne", "ネ" => "ne",
            "の" => "no", "ノ" => "no",
            "は" => "ha", "ハ" => "ha", "ひ" => "hi", "ヒ" => "hi", "ふ" => "fu", "フ" => "fu",
            "へ" => "he", "ヘ" => "he", "ほ" => "ho", "ホ" => "ho",
            "ま" => "ma", "マ" => "ma", "み" => "mi", "ミ" => "mi", "む" => "mu", "ム" => "mu",
            "め" => "me", "メ" => "me", "も" => "mo", "モ" => "mo",
            "や" => "ya", "ヤ" => "ya", "ゆ" => "yu", "ユ" => "yu", "よ" => "yo", "ヨ" => "yo",
            "ら" => "ra", "ラ" => "ra", "り" => "ri", "リ" => "ri", "る" => "ru", "ル" => "ru",
            "れ" => "re", "レ" => "re", "ろ" => "ro", "ロ" => "ro",
            "わ" => "wa", "ワ" => "wa", "ゐ" => "i", "を" => "o", "ヲ" => "o", "ん" => "n", "ン" => "n",
            "が" => "ga", "ガ" => "ga", "ぎ" => "gi", "ギ" => "gi", "ぐ" => "gu", "グ" => "gu",
            "げ" => "ge", "ゲ" => "ge", "ご" => "go", "ゴ" => "go", "ざ" => "za", "ザ" => "za",
            "じ" => "ji", "ジ" => "ji", "ず" => "zu", "ズ" => "zu",
            "ぜ" => "ze", "ゼ" => "ze", "ぞ" => "zo", "ゾ" => "zo",
            "だ" => "da", "ダ" => "da", "ぢ" => "ji", "ヂ" => "ji", "づ" => "zu", "ヅ" => "zu",
            "で" => "de", "デ" => "de", "ど" => "do", "ド" => "do",
            "ば" => "ba", "バ" => "ba", "び" => "bi", "ビ" => "bi", "ぶ" => "bu", "ブ" => "bu",
            "べ" => "be", "ベ" => "be", "ぼ" => "bo", "ボ" => "bo",
            "ぱ" => "pa", "パ" => "pa", "ぴ" => "pi", "ピ" => "pi", "ぷ" => "pu", "プ" => "pu",
            "ぺ" => "pe", "ペ" => "pe", "ぽ" => "po", "ポ" => "po",
            "きゃ" => "kya", "キャ" => "kya", "きゅ" => "kyu", "キュ" => "kyu",
            "きょ" => "kyo", "キョ" => "kyo",
            "しゃ" => "sha", "シャ" => "sha", "しゅ" => "shu", "シュ" => "shu",
            "しょ" => "sho", "ショ" => "sho",
            "ちゃ" => "cha", "チャ" => "cha", "ちゅ" => "chu", "チュ" => "chu",
            "ちょ" => "cho", "チョ" => "cho",
            "にゃ" => "nya", "ニャ" => "nya", "にゅ" => "nyu", "ニュ" => "nyu",
            "にょ" => "nyo", "ニョ" => "nyo",
            "ひゃ" => "hya", "ヒャ" => "hya", "ひゅ" => "hyu", "ヒュ" => "hyu",
            "ひょ" => "hyo", "ヒョ" => "hyo",
            "みゃ" => "mya", "ミャ" => "mya", "みゅ" => "myu", "ミュ" => "myu",
            "みょ" => "myo", "ミョ" => "myo",
            "りゃ" => "rya", "リャ" => "rya", "りゅ" => "ryu", "リュ" => "ryu",
            "りょ" => "ryo", "リョ" => "ryo",
            "ぎゃ" => "gya", "ギャ" => "gya", "ぎゅ" => "gyu", "ギュ" => "gyu",
            "ぎょ" => "gyo", "ギョ" => "gyo",
            "じゃ" => "ja", "ジャ" => "ja", "じゅ" => "ju", "ジュ" => "ju",
            "じょ" => "jo", "ジョ" => "jo",
            "びゃ" => "bya", "ビャ" => "bya", "びゅ" => "byu", "ビュ" => "byu",
            "びょ" => "byo", "ビョ" => "byo",
            "ぴゃ" => "pya", "ピャ" => "pya", "ぴゅ" => "pyu", "ピュ" => "pyu",
            "ぴょ" => "pyo", "ピョ" => "pyo",
            "〜" => "~",
            "っ" => "_", "ッ" => "_",
            "ー" => "-",
            _ => "",
        }.to_string();
        match temp.as_str(){
            "" => Self::Fail,
            "_" => Self::SmallTsu,
            "-" => Self::Enlongate,
            _ => Self::Roman(temp),
        }
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_romanize(){
        assert_eq!(
            &romanize("ちょうしょく は じぶんで!! つくって ください!!"),
            "choushoku ha jibunde!! tsukutte kudasai!!"
        );
        assert_eq!(
            &romanize("いってキーまーす!!"),
            "ittekiimaasu!!"
        );
    }
}
