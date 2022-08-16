use crate::structure::*;
use crate::japanese::*;
use crate::report::*;

use std::fmt::Write;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct LangStats{
    rp: ReportHeader,
    kanji: HashMap<String, usize>,
    other: HashMap<String, usize>,
}

pub fn lang_stats_report(mut s: LangStats, doc: &mut String){
    write_header(&mut s.rp, doc);

    write_list(s.other, "Hiragana/Katakana frequencies:", doc);
    write_list(s.kanji, "Kanji frequencies:", doc);
}

pub fn accumulate_lang_stats(chapter: Chapter, stats: &mut LangStats, log: &mut String){
    set_current_manga(&mut stats.rp.manga, chapter.manga, log);
    stats.rp.volumes.push(chapter.volume);
    stats.rp.chapters.push(chapter.chapter);
    for picture in chapter.pic{
        stats.rp.pictures += 1;

        if let Some(texts) = picture.text{
            for text in texts{
                let replacements = if let Some(kmap) = &text.kmap{
                    for [kanji, mapping] in kmap{
                        let key = format!("{}: {}", kanji, mapping);
                        update(&mut stats.kanji, &key, 1, |a, b| a + b);
                    }
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
                for line in text.lines{
                    for c in line.chars(){
                        if is_hiragana(c) || is_katakana(c) {
                            update(&mut stats.other, &c.to_string(), 1, |a, b| a + b);
                        }
                    }
                }
                stats.rp.morae += morae;
            }
        };
    }
}

