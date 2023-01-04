use crate::structure::*;
use crate::japanese::*;
use crate::report::*;

use std::fmt::Write;

pub fn write_transcription(chapter: Chapter, md: &mut String, log: &mut String){
    let _ = writeln!(md, "{}{}", header(1), &chapter.title);
    let _ = writeln!(md, "Manga: {}", chapter.manga);
    let _ = writeln!(md, "Author: {}", chapter.author);
    let _ = writeln!(md, "Volume: {}", chapter.volume);
    let _ = writeln!(md, "Chapter: {}", chapter.chapter);
    if let Some(subchap) = chapter.subchapter{
        let _ = writeln!(md, "Sub Chapter: {}", subchap);
    }

    let mut page = 0;
    let mut pic_nr = 1;
    let mut last_written_page = 0;

    if chapter.pic.is_empty() { return; }
    chapter_header_log(&chapter, log);

    for picture in chapter.pic{
        fn write_text(md: &mut String, log: &mut String, ident: usize, text: Text){
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
            log_todo(&text, log);
            let lines = text.lines.vectorize();
            // transcription
            let _ = write!(md, "{}", bullet(ident + 1));
            write_lines(md, &lines, &[
                (" ", ""), ("-", "ー"), ("~", "〜"), ("!", "！"), ("?", "？")
            ]);
            let _ = writeln!(md);
            // kanji replacement
            let replacements = if let Some(kmap) = text.kmap{
                let rs = map_kanjis(&lines, kmap.vectorize().as_slice());
                let _ = write!(md, "{}", bullet(ident + 1));
                write_lines(md, &rs, &[(" ", "")]);
                let _ = writeln!(md);
                rs
            } else {
                lines.clone()
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
            if let Some(transl) = text.transl{
                let _ = write!(md, "{}", bullet(ident + 1));
                write_lines(md, &transl.vectorize(), &[]);
                let _ = writeln!(md);
            }
            // notes
            if let Some(notes) = text.notes{
                let _ = write!(md, "{}", bullet(ident + 1));
                write_lines(md, &notes.vectorize(), &[]);
                let _ = writeln!(md);
            }
        }

        pic_nr = picture.nr.unwrap_or(pic_nr + 1);
        page = picture.page.unwrap_or(page);

        let text = if let Some(text) = picture.text{ text } else { continue; };

        if page > last_written_page{
            last_written_page = page;
            let _ = writeln!(md, "{}Page: {}", header(5), page);
        }
        let _ = writeln!(md, "{}picture {}", bullet(0), pic_nr);

        let multiple = text.len() > 1;
        let ident = usize::from(multiple);
        for (n, text) in text.into_iter().enumerate(){
            if multiple{
                let _ = writeln!(md, "{}text {}", bullet(1), n + 1);
            }
            write_text(md, log, ident, text);
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

