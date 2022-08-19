use crate::structure::*;
use crate::japanese::*;
use crate::report::*;

use std::fmt::Write;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Stats{
    rp: ReportHeader,
    locations: HashMap<String, (usize, usize)>,
    characters: HashMap<String, usize>,
    speaks: HashMap<String, usize>,
    spoken_to: HashMap<String, usize>,
    conversation_pair: HashMap<String, usize>,
    conversation_prominence: HashMap<String, usize>
}

pub fn stats_report(mut s: Stats, doc: &mut String){
    write_header(&mut s.rp, doc);

    let _ = writeln!(doc, "Locations: ");
    let mut locs = s.locations.into_iter().collect::<Vec<_>>();
    locs.sort_unstable_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap());
    for (name, (count, morae)) in locs{
        let _ = writeln!(doc, "\t{}: {} appearances, {} morae spoken in.", name, count, morae);
    }

    let total_appearances = write_list(&s.characters, "Character appearances:", "", doc) as f64;
    write_list(&s.speaks, "Morae spoken:", "", doc);
    write_list(&s.spoken_to, "Morae spoken to:", "", doc);
    write_list(&s.conversation_pair, "Conversation pairs in morae:", "", doc);

    let total_prom: usize = s.conversation_prominence.iter().map(|(_, c)| c).sum();
    let mut prom = s.conversation_prominence.into_iter()
        .map(|(s, c)| (s, c as f64 / total_prom as f64))
        .collect::<HashMap<_, f64>>();

    for (character, count) in &s.characters{
        let val = *count as f64 / total_appearances;
        update(&mut prom, character, |x| (x + val));
    }
    prom.iter_mut().for_each(|(_, c)| *c *= 50.0);
    write_list(&prom, "Character prominence:", "%", doc);
}

pub fn accumulate_stats(chapter: Chapter, stats: &mut Stats, log: &mut String){
    set_current_manga(&mut stats.rp.manga, chapter.manga, log);
    stats.rp.volumes.push(chapter.volume);
    stats.rp.chapters.push(chapter.chapter);
    for picture in chapter.pic{
        stats.rp.pictures += 1;
        let location = picture.location.unwrap_or_default();
        let mut pic_morae = 0;
        if let Some(characters) = picture.characters{
            for character in characters{
                update(&mut stats.characters, &character, |x| x + 1);
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
                stats.rp.morae += morae;
                pic_morae += morae;
                update(&mut stats.speaks, &text.from, |x| x + morae);
                update(&mut stats.conversation_prominence, &text.from, |x| x + morae);
                if let Some(receiver) = text.to{
                    let pair = if text.from.cmp(&receiver) == std::cmp::Ordering::Less{
                        format!("{}, {}", text.from, receiver)
                    } else {
                        format!("{}, {}", receiver, text.from)
                    };
                    update(&mut stats.spoken_to, &receiver, |x| x + morae);
                    update(&mut stats.conversation_pair, &pair, |x| x + morae);
                    update(&mut stats.conversation_prominence, &receiver, |x| x + morae);
                }
            }
        };
        update(&mut stats.locations, &location, |(a, b)| (a + 1, b + pic_morae));
    }
}

