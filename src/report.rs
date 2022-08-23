use crate::Chapter;

use std::fmt::Write;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct ReportHeader{
    pub manga: String,
    pub volumes: Vec<usize>,
    pub chapters: Vec<usize>,
    pub pictures: usize,
    pub morae: usize,
}

pub fn write_header(h: &mut ReportHeader, doc: &mut String){
    let _ = writeln!(doc, "Manga: {}", h.manga);
    let _ = write!(doc, "Volumes: ");

    h.volumes.sort();
    h.volumes.dedup();
    for vol in &h.volumes{
        let _ = write!(doc, "{}, ", vol);
    }
    doc.pop();
    doc.pop();
    let _ = writeln!(doc);

    let _ = write!(doc, "Chapters: ");
    h.chapters.sort();
    h.chapters.dedup();
    for chap in &h.chapters{
        let _ = write!(doc, "{}, ", chap);
    }
    doc.pop();
    doc.pop();
    let _ = writeln!(doc);

    let _ = writeln!(doc, "Pictures: {}", h.pictures);
    let _ = writeln!(doc, "Morae spoken: {}", h.morae);
}

pub fn write_list<'a,T>(col: &'a HashMap<String, T>, title: &str, unit: &str, doc: &mut String) -> T
    where T: std::fmt::Display + PartialOrd + Copy + std::iter::Sum<&'a T> + 'a
{
    let total: T = col.iter().map(|(_, c)| c).sum();
    let _ = writeln!(doc, "{} (out of {:.2}{})", title, total, unit);
    let mut list = col.iter().collect::<Vec<_>>();
    list.sort_unstable_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap());
    for (name, count) in list{
        let _ = writeln!(doc, "\t{}: {:.2}{}", name, count, unit);
    }
    total
}


pub fn update<T: Copy + Default + Sized, F>(map: &mut HashMap<String, T>, key: &str, fun: F)
    where F: Fn(T) -> T
{
    if let Some(x) = map.get_mut(key){
        *x = fun(*x);
    } else {
        map.insert(key.to_string(), fun(T::default()));
    }
}

pub fn set_current_manga(current: &mut String, mut chapter: String, log: &mut String){
    if current.is_empty(){
        *current = chapter;
    } else if current != &mut chapter{
        let _ = writeln!(log, "Different manga found: {}. Current manga is: {}.", chapter, current);
    }
}

pub fn chapter_header_log(chapter: &Chapter, log: &mut String){
    let warning_header = format!(
        "Warning: chapter {} of volume {} of manga {}",
        chapter.chapter, chapter.volume, chapter.manga,
    );
    if chapter.pic[0].location.is_none() {
        let _ = writeln!(
            log,
            "{} does not have a location set in it's first picture.",
            warning_header
        );
    }
    if chapter.pic[0].page.is_none() {
        let _ = writeln!(
            log, "{} does not have a page number set in it's first picture.",
            warning_header
        );
    }
}
