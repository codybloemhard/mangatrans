use serde::Deserialize;
use clap::Parser;

use std::fs;
use std::fmt::Write;
use std::io::Write as _;
use std::path::PathBuf;
use std::collections::HashMap;

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
    #[clap(short='m', long, value_enum, default_value_t=Mode::default())]
    mode: Mode,
    #[clap(short='o', long, value_enum, default_value_t=OutputMode::default())]
    outputmode: OutputMode,
    #[clap(short='l', long, default_value_t=true)]
    log: bool,
    #[clap(short='d', long, value_parser)]
    outputdir: Option<PathBuf>,
    #[clap(required = true)]
    inputfiles: Vec<PathBuf>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, clap::ValueEnum)]
enum Mode { #[default] Transcribe, Stats }

#[derive(Debug, Default, Clone, Copy, PartialEq, clap::ValueEnum)]
enum OutputMode { #[default] Stdout, File }

fn main() {
    let args = Args::parse();
    if args.inputfiles.is_empty(){
        println!("No input files received!");
        return;
    }
    let mut log = String::new();
    let mut doc = String::new();
    let mut stats = Stats::default();
    let mut fileroot = args.inputfiles[0].clone();
    for file in args.inputfiles{
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
        if args.mode == Mode::Transcribe{
            doc.clear();
            write_transcription(chapter, &mut doc, &mut log);
            write_output(args.outputmode, &args.outputdir, file, &doc);
        } else {
            accumulate_stats(chapter, &mut stats, &mut log);
        }
    }
    if args.mode == Mode::Stats{
        fileroot.set_file_name("stats");
        stats_report(stats, &mut doc);
        write_output(args.outputmode, &args.outputdir, fileroot, &doc);
    }
    if args.log {
        println!("{}", log);
    }
}

#[derive(Debug, Clone, Default)]
struct Stats{
    manga: String,
    volumes: Vec<usize>,
    chapters: Vec<usize>,
    pictures: usize,
    morae: usize,
    locations: HashMap<String, (usize, usize)>,
    characters: HashMap<String, usize>,
    speaks: HashMap<String, usize>,
    spoken_to: HashMap<String, usize>,
    conversation_pair: HashMap<String, usize>,
}

fn stats_report(mut s: Stats, doc: &mut String){
    let _ = writeln!(doc, "Manga: {}", s.manga);
    let _ = write!(doc, "Volumes: ");

    s.volumes.sort();
    s.volumes.dedup();
    for vol in s.volumes{
        let _ = write!(doc, "{}, ", vol);
    }
    doc.pop();
    doc.pop();
    let _ = writeln!(doc);

    let _ = write!(doc, "Chapters: ");
    s.chapters.sort();
    s.chapters.dedup();
    for chap in s.chapters{
        let _ = write!(doc, "{}, ", chap);
    }
    doc.pop();
    doc.pop();
    let _ = writeln!(doc);

    let _ = writeln!(doc, "Pictures: {}", s.pictures);
    let _ = writeln!(doc, "Morae spoken: {}", s.morae);

    let _ = writeln!(doc, "Locations: ");
    let mut locs = s.locations.into_iter().collect::<Vec<_>>();
    locs.sort_unstable_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap());
    for (name, (count, morae)) in locs{
        let _ = writeln!(doc, "\t{}: {} appearances, {} morae spoken in.", name, count, morae);
    }

    let mut write_list = |hmap: HashMap<String, usize>, title: &str|{
        let _ = writeln!(doc, "{}", title);
        let mut list = hmap.into_iter().collect::<Vec<_>>();
        list.sort_unstable_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap());
        for (name, count) in list{
            let _ = writeln!(doc, "\t{}: {}", name, count);
        }
    };

    write_list(s.characters, "Character appearances: ");
    write_list(s.speaks, "Morae spoken: ");
    write_list(s.spoken_to, "Morae spoken to: ");
    write_list(s.conversation_pair, "Conversation pairs in Morae: ");
}

fn accumulate_stats(chapter: Chapter, stats: &mut Stats, log: &mut String){
    fn update<T: Copy>(map: &mut HashMap<String, T>, key: &str, val: T, fun: fn(T, T) -> T){
        if let Some(x) = map.get_mut(key){
            *x = fun(*x, val);
        } else {
            map.insert(key.to_string(), val);
        }
    }

    if stats.manga.is_empty(){
        stats.manga = chapter.manga;
    } else if stats.manga != chapter.manga{
        let _ = writeln!(log, "Different manga found: {}. Current manga is: {}.",
                         chapter.manga, stats.manga);
    }
    stats.volumes.push(chapter.volume);
    stats.chapters.push(chapter.chapter);
    for picture in chapter.pic{
        stats.pictures += 1;
        let location = picture.location.unwrap_or_default();
        let mut pic_morae = 0;
        if let Some(characters) = picture.characters{
            for character in characters{
                update(&mut stats.characters, &character, 1, |a, b| a + b);
            }
        }
        if let Some(texts) = picture.text{
            for text in texts{
                let replacements = if let Some(kmap) = &text.kmap{
                    map_kanjis(&text.lines, kmap.as_slice())
                } else {
                    text.lines.clone()
                };
                if could_contain_kanji(&replacements){
                    let _ = writeln!(
                        log,
                        "Warning: lines {:#?} contain kanji or untranslateable characters.
                        Every kanji is counted as one (1) mora.",
                        replacements
                    );
                }
                let morae = replacements.iter().flat_map(|line| line.chars())
                    .fold(0, |acc, c| acc + to_mora(c));
                stats.morae += morae;
                pic_morae += morae;
                update(&mut stats.speaks, &text.from, morae, |a, b| a + b);
                if let Some(receiver) = text.to{
                    let pair = if text.from.cmp(&receiver) == std::cmp::Ordering::Less{
                        format!("{}, {}", text.from, receiver)
                    } else {
                        format!("{}, {}", receiver, text.from)
                    };
                    update(&mut stats.spoken_to, &receiver, morae, |a, b| a + b);
                    update(&mut stats.conversation_pair, &pair, morae, |a, b| a + b);
                }
            }
        };
        update(&mut stats.locations, &location, (1, pic_morae), |(a, b), (c, d)| (a + c, b + d));
    }
}

fn write_output(outputmode: OutputMode, outputdir: &Option<PathBuf>, mut file: PathBuf, doc: &str){
    if outputmode == OutputMode::File{
        if let Some(outdir) = outputdir{
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
                return;
            }
        };
        if let Err(error) = write!(outfile, "{}", doc){
            println!("Could not write to file: \"{}\".\n\tError: {}", file.display(), error);
        }
    } else {
        println!("{}", doc);
    }
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
            "〜" => "~", "？" => "?", "！" => "!", "　" => " ", "「" => "\"", "」" => "\"",
            "。" => ".", "、" => ",",
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
    !is_latin(c) && !is_hiragana(c) && !is_katakana(c) && !is_punctuation(c) && !is_whitespace(c)
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
    "-_=+`~,./<>?\\|[]{}!@#$%^&*()〜ー！？・「」、。".contains(c)
}

fn is_whitespace(c: char) -> bool{
    " 　\t\n".contains(c)
}

fn to_mora(c: char) -> usize{
    if "ゃゅょャュョ 　〜！？・「」、。-_=+`~,./<>?\\|[]{}!@#$%^&*(\"'".contains(c) { return 0; }
    if is_latin(c) { return 0; }
    if is_whitespace(c) { return 0; }
    1
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

    #[test]
    fn to_mora_test(){
        fn morae(string: &str) -> usize{
            string.chars().fold(0, |a, c| a + to_mora(c))
        }
        assert_eq!(morae("きょう"), 2);
        assert_eq!(morae("  いってキーまーす！"), 8);
    }
}
