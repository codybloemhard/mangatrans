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


pub fn write_list(hmap: HashMap<String, usize>, title: &str, doc: &mut String){
    let _ = writeln!(doc, "{}", title);
    let mut list = hmap.into_iter().collect::<Vec<_>>();
    list.sort_unstable_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap());
    for (name, count) in list{
        let _ = writeln!(doc, "\t{}: {}", name, count);
    }
}

pub fn update<T: Copy>(map: &mut HashMap<String, T>, key: &str, val: T, fun: fn(T, T) -> T){
    if let Some(x) = map.get_mut(key){
        *x = fun(*x, val);
    } else {
        map.insert(key.to_string(), val);
    }
}

pub fn set_current_manga(current: &mut String, mut chapter: String, log: &mut String){
    if current.is_empty(){
        *current = chapter;
    } else if current != &mut chapter{
        let _ = writeln!(log, "Different manga found: {}. Current manga is: {}.", chapter, current);
    }
}
