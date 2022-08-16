use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Chapter{
    pub manga: String,
    pub author: String,
    pub volume: usize,
    pub chapter: usize,
    pub subchapter: Option<f32>,
    pub title: String,
    pub pic: Vec<Pic>,
}

#[derive(Deserialize, Debug)]
pub struct Pic{
    pub nr: usize,
    pub page: usize,
    pub characters: Option<Vec<String>>,
    pub location: Option<String>,
    pub text: Option<Vec<Text>>,
}

#[derive(Deserialize, Debug)]
pub struct Text{
    pub from: String,
    pub to: Option<String>,
    pub lines: Vec<String>,
    pub kmap: Option<Vec<[String; 2]>>,
    pub transl: Option<Vec<String>>,
}

