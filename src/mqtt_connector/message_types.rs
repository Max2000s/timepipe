use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TimeseriesSchema {
    pub version: String,
    pub elements: Vec<SchemaElement>,
}

#[derive(Debug, Deserialize)]
pub struct SchemaElement {
    pub name: String,
    pub description: String,
    pub unit: String,
}

#[derive(Debug, Deserialize)]
pub struct TimeseriesData {
    pub version: String,
    pub timestamp: String,
    pub values: Vec<f64>,
}
