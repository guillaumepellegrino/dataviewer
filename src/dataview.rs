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

/// The root definition of a DataView File
#[derive(Debug, PartialEq, Default, Clone, Deserialize, Serialize)]
pub struct File {
    #[serde(default)]
    pub dataview: DataView,

    #[serde(default)]
    pub chart: HashMap<String, Chart>,

    #[serde(default)]
    pub data: HashMap<String, Vec<f64>>,
}
