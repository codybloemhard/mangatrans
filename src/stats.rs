use crate::structure::*;
use crate::japanese::*;

use std::fmt::Write;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Stats{
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

pub fn stats_report(mut s: Stats, doc: &mut String){
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

pub fn accumulate_stats(chapter: Chapter, stats: &mut Stats, log: &mut String){
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

