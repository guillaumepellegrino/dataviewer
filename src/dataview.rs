/// Definition of the DataView File Format
use std::collections::HashMap;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, PartialEq, Default, Copy, Clone, Deserialize, Serialize)]
pub enum Type {
    #[default]
    XY,
    Line,
}

#[derive(Debug, PartialEq, Default, Clone, Deserialize, Serialize)]
pub struct DataView {
    pub r#type: Type,
    pub title: Option<String>,
    pub x_title: Option<String>,
    pub y_title: Option<String>,
    pub x_unit: Option<String>,
    pub y_unit: Option<String>,
    pub x_min: Option<f64>,
    pub x_max: Option<f64>,
    pub y_min: Option<f64>,
    pub y_max: Option<f64>,
    pub description: Option<String>
}

#[derive(Debug, PartialEq, Default, Clone, Deserialize, Serialize)]
pub struct Chart {
    pub title: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, PartialEq, Default, Clone, Deserialize, Serialize)]
pub struct Data {
    pub data: Vec<f64>,
}

/// The root definition of a DataView File
#[derive(Debug, PartialEq, Default, Clone, Deserialize, Serialize)]
pub struct File {
    pub dataview: DataView,
    pub chart: HashMap<String, Chart>,
    pub data: HashMap<String, Data>,
}




impl Data {
    pub fn pair_iter(&self) -> PairIterator {
        PairIterator {
            iter: self.data.iter()
        }
    }
}

pub struct PairIterator<'a> {
    iter: std::slice::Iter<'a, f64>,
}

impl<'a> Iterator for PairIterator<'a> {
    type Item = (f64, f64);

    fn next(&mut self) -> Option<Self::Item> {
        let a = match self.iter.next() {
            Some(a) => a,
            None => {return None;},
        };
        let b = match self.iter.next() {
            Some(b) => b,
            None => {return None;},
        };
        Some((*a, *b))
    }
}
