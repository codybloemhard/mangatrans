use serde::Deserialize;
use clap::Parser;

use std::fs;
use std::fmt::Write;
use std::io::Write as _;

#[derive(Deserialize, Debug)]
struct Chapter{
    manga: String,
    author: String,
    volume: usize,
    chapter: usize,
    subchapter: Option<f32>,
    title: String,
    pic: Vec<Pic>,
}

#[derive(Deserialize, Debug)]
struct Pic{
    nr: usize,
    page: usize,
    characters: Option<Vec<String>>,
    location: Option<String>,
    text: Option<Vec<Text>>,
}

#[derive(Deserialize, Debug)]
struct Text{
    from: String,
    to: Option<String>,
    lines: Vec<String>,
    kmap: Option<Vec<[String; 2]>>,
    transl: Option<Vec<String>>,
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args{
    #[clap(short='o', long, value_enum, default_value_t=OutputMode::default())]
    outputmode: OutputMode,
    #[clap(short='d', long, value_parser)]
    outputdir: Option<std::path::PathBuf>,
    #[clap(short, long, value_parser)]
    inputfiles: Vec<std::path::PathBuf>,
}

#[derive(Debug, Default, Clone, PartialEq, clap::ValueEnum)]
enum OutputMode { #[default] Stdout, File }

fn main() {
    let args = Args::parse();
    let mut log = String::new();
    let mut md = String::new();
    for mut file in args.inputfiles{
        let contents = match fs::read_to_string(&file){
            Ok(contents) => contents,
            Err(error) => {
                println!("Could not read file: \"{}\".\n\tError: {}", file.display(), error);
                continue;
            }
        };
        let chapter = match toml::from_str::<Chapter>(&contents){
            Ok(chapter) => chapter,
            Err(error) => panic!("{} (error position is an estimation!)", error),
        };
        write_transcription(chapter, &mut md, &mut log);
        if args.outputmode == OutputMode::File{
            if let Some(outdir) = &args.outputdir{
                let filename = file.file_name().expect("rip").to_os_string();
                file.clear();
                file.push(outdir);
                file.push(filename);
            }
            file.set_extension("md");
            let mut outfile = match fs::File::create(&file){
                Ok(outfile) => outfile,
                Err(error) => {
                    println!("Could not create file: \"{}\".\n\tError: {}", file.display(), error);
                    continue;
                }
            };
            if let Err(error) = write!(outfile, "{}", md){
                println!("Could not write to file: \"{}\".\n\tError: {}", file.display(), error);
            }
        } else {
            println!("{}", md);
        }
    }
    println!("{}", log);
}

fn write_transcription(chapter: Chapter, md: &mut String, log: &mut String){
    let _ = writeln!(md, "{}{}", header(1), &chapter.title);
    let _ = writeln!(md, "Manga: {}", chapter.manga);
    let _ = writeln!(md, "Author: {}", chapter.author);
    let _ = writeln!(md, "Volume: {}", chapter.volume);
    let _ = writeln!(md, "Chapter: {}", chapter.chapter);
    if let Some(subchap) = chapter.subchapter{
        let _ = writeln!(md, "Sub Chapter: {}", subchap);
    }

    let mut last_page = 0;
    for picture in chapter.pic{
        fn write_text(md: &mut String, log: &mut String, ident: usize, text: &Text){
            fn write_lines(md: &mut String, lines: &[String], reps: &[(&str, &str)]) {
                for line in lines{
                    let mut newline = line.clone();
                    for (replacee, replacant) in reps{
                        newline = newline.replace(replacee, replacant);
                    }
                    let _ = write!(md, "{} <br/> ", newline);
                }
                for _ in 0..7 { md.pop(); }
            }
            // transcription
            let _ = write!(md, "{}", bullet(ident + 1));
            write_lines(md, &text.lines, &[
                (" ", ""), ("-", "ー"), ("~", "〜"), ("!", "！"), ("?", "？")
            ]);
            let _ = writeln!(md);
            // kanji replacement
            let replacements = if let Some(kmap) = &text.kmap{
                let rs = map_kanjis(&text.lines, kmap.as_slice());
                let _ = write!(md, "{}", bullet(ident + 1));
                write_lines(md, &rs, &[(" ", "")]);
                let _ = writeln!(md);
                rs
            } else {
                text.lines.clone()
            };
            // romanize
            if could_contain_kanji(&replacements){
                let _ = writeln!(
                    log,
                    "Warning: lines {:#?} contain kanji or untranslateable characters.",
                    replacements
                );
            } else {
                let mut romanizeds = Vec::new();
                for rep in &replacements{
                    romanizeds.push(romanize(rep));
                }
                let _ = write!(md, "{}", bullet(ident + 1));
                write_lines(md, &romanizeds, &[
                    ("　", " "), ("ー", "-"), ("〜", "~"), ("！", "!"), ("？", "?")
                ]);
                let _ = writeln!(md);
            }
            // translation
            if let Some(transl) = &text.transl{
                let _ = write!(md, "{}", bullet(ident + 1));
                write_lines(md, transl, &[]);
                let _ = writeln!(md);
            }
        }

        let text = if let Some(text) = picture.text{ text } else { continue; };

        if picture.page > last_page{
            let _ = writeln!(md, "{}Page: {}", header(5), picture.page);
            last_page = picture.page;
        }
        let _ = writeln!(md, "{}picture {}", bullet(0), picture.nr);

        if text.len() == 1{
            write_text(md, log, 1, &text[0]);
        } else {
            for (n, text) in text.iter().enumerate(){
                let _ = writeln!(md, "{}text {}", bullet(1), n + 1);
                write_text(md, log, 1, text);
            }
        }
    }
}

fn header(rank: usize) -> String{
    let mut temp = "#".repeat(rank);
    temp.push(' ');
    temp
}

fn bullet(ident: usize) -> String{
    let mut temp = "  ".repeat(ident);
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
        if let Hepburn::Roman(roman) = Hepburn::from(&comb){
            push(&mut res, &roman, &mut tsu, &mut prev);
            i += 2;
            continue;
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
        if last == "っ"{
            res.push('h');
        } else if let Hepburn::Roman(roman) = Hepburn::from(&last){
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
            "〜" => "~", "？" => "?", "！" => "!", "　" => " ",
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

fn map_kanjis(strings: &[String], subs: &[[String; 2]]) -> Vec<String>{
    let mut replaceds = strings.to_vec();
    for [replacee, replacant] in subs{
        for (i, string) in replaceds.iter().enumerate(){
            let new = string.replacen(replacee, replacant, 1);
            if &new != string {
                replaceds[i] = new;
                break;
            }
        }
    }
    replaceds
}

fn could_contain_kanji(strings: &[String]) -> bool{
    for string in strings{
        for c in string.chars(){
            if could_be_kanji(c) { return true; }
        }
    }
    false
}

fn could_be_kanji(c: char) -> bool{
    !is_latin(c) && !is_hiragana(c) && !is_katakana(c) && !is_punctuation(c)
}

fn is_latin(c: char) -> bool{
    "qgmlwyfubdstnriaeohzxcvjkpQGMLWYFUBDSTNRIAEOHZXCVJKP0123456789".contains(c)
}

fn is_hiragana(c: char) -> bool{
    "あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろ
    わをんがぎぐげござじずぜぞだぢづでどばびぶべぼぱぴぷぺぽゐゃゅょっ".contains(c)
}

fn is_katakana(c: char) -> bool{
    "アイウエオカキクケコサシスセソタチツテトナニヌネノハヒフヘホマミムメモヤユヨラリルレロ
    ワヲンガギグゲゴザジズゼゾダヂヅデドバビブベボパピプペポャュョッ".contains(c)
}

fn is_punctuation(c: char) -> bool{
    "-_=+`~,./<>?\\|[]{}!@#$%^&*() 　〜ー！？・".contains(c)
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

    #[test]
    fn test_map_kanjis(){
        // normal one of each
        let map = vec!
            [
                ("今", "きょ"), ("日", "う"), ("雨", "あめ"), ("降", "ふ"),
                ("作", "つく"), ("下", "くだ")
            ]
            .into_iter().map(|(a, b)| [a.to_string(), b.to_string()]).collect::<Vec<_>>();
        assert_eq!(
            map_kanjis(&[
                "今日雨降るって".to_string(),
                "作って 下さい!!".to_string()
            ], &map),
            vec![
                "きょうあめふるって".to_string(),
                "つくって ください!!".to_string()
            ]
        );
        // more than kanji mapping in one map entry
        let map = vec!
            [
                ("今日", "きょう"), ("雨", "あめ"), ("降", "ふ"), ("作", "つく"), ("下", "くだ")
            ]
            .into_iter().map(|(a, b)| [a.to_string(), b.to_string()]).collect::<Vec<_>>();
        assert_eq!(
            map_kanjis(&[
                "今日雨降るって".to_string(),
                "作って 下さい!!".to_string()
            ], &map),
            vec![
                "きょうあめふるって".to_string(),
                "つくって ください!!".to_string()
            ]
        );
        // two of the same in one line
        let map = vec!
            [
                ("考", "かんが"), ("不", "ふ"), ("幸", "こう"), ("中", "ちゅ"), ("幸", "さいわ"),
            ]
            .into_iter().map(|(a, b)| [a.to_string(), b.to_string()]).collect::<Vec<_>>();
        assert_eq!(
            map_kanjis(&[
                "そう考えると".to_string(),
                "不幸中の幸いって".to_string(),
                "ヤツだね".to_string()
            ], &map),
            vec![
                "そうかんがえると".to_string(),
                "ふこうちゅのさいわいって".to_string(),
                "ヤツだね".to_string()
            ]
        );
        // two of the same in two lines
        let map = vec!
            [
                ("考", "かんが"), ("不", "ふ"), ("幸", "こう"), ("中", "ちゅ"), ("幸", "さいわ"),
            ]
            .into_iter().map(|(a, b)| [a.to_string(), b.to_string()]).collect::<Vec<_>>();
        assert_eq!(
            map_kanjis(&[
                "そう考えると".to_string(),
                "不幸中の".to_string(),
                "幸いって".to_string(),
                "ヤツだね".to_string()
            ], &map),
            vec![
                "そうかんがえると".to_string(),
                "ふこうちゅの".to_string(),
                "さいわいって".to_string(),
                "ヤツだね".to_string()
            ]
        );
        // all of it
        let map = vec!
            [
                ("今日", "きょう"), ("考", "かんが"), ("不", "ふ"),
                ("幸", "こう"), ("中", "ちゅ"), ("幸", "さいわ"), ("幸", "justatest")
            ]
            .into_iter().map(|(a, b)| [a.to_string(), b.to_string()]).collect::<Vec<_>>();
        assert_eq!(
            map_kanjis(&[
                "今日考".to_string(),
                "不".to_string(),
                "幸幸中".to_string(),
                "幸".to_string()
            ], &map),
            vec![
                "きょうかんが".to_string(),
                "ふ".to_string(),
                "こうさいわちゅ".to_string(),
                "justatest".to_string()
            ]
        );
    }
}
