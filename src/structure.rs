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
    pub nr: Option<usize>,
    pub page: Option<usize>,
    pub characters: Option<Vec<String>>,
    pub location: Option<String>,
    pub text: Option<Vec<Text>>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Either<T, U>{ This(T), That(U) }

#[derive(Deserialize, Debug)]
pub struct Text{
    pub from: Either<String, Vec<String>>,
    pub to: Option<Either<String, Vec<String>>>,
    pub todo: Option<bool>,
    pub lines: Vec<String>,
    pub kmap: Option<Vec<[String; 2]>>,
    pub transl: Option<Vec<String>>,
    pub notes: Option<Vec<String>>,
}

