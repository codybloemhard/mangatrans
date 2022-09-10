use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Either<T, U>{ This(T), That(U) }

pub type OneOrMore<T> = Either<T, Vec<T>>;

pub trait Vectorize<T>{
    fn vectorize(self) -> Vec<T>;
}

impl<T> Vectorize<T> for OneOrMore<T>{
    fn vectorize(self) -> Vec<T>{
        match self{
            Either::This(x) => vec![x],
            Either::That(x) => x,
        }
    }
}

impl<T, U> Vectorize<T> for Option<U> where U: Vectorize<T>{
    fn vectorize(self) -> Vec<T>{
        match self{
            Some(x) => x.vectorize(),
            None => vec![],
        }
    }
}

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
    pub characters: Option<OneOrMore<String>>,
    pub location: Option<String>,
    pub text: Option<Vec<Text>>,
}

#[derive(Deserialize, Debug)]
pub struct Text{
    pub from: OneOrMore<String>,
    pub to: Option<OneOrMore<String>>,
    pub todo: Option<bool>,
    pub lines: OneOrMore<String>,
    pub kmap: Option<OneOrMore<[String; 2]>>,
    pub transl: Option<OneOrMore<String>>,
    pub notes: Option<OneOrMore<String>>,
}

