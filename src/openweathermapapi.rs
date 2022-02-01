use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct OneCall {
    pub lat: f64,
    pub lon: f64,
    pub timezone_offset: i64,
    pub current: Option<Current>,
    pub daily: Option<Vec<Daily>>,
}

#[derive(Serialize, Deserialize)]
pub struct Current {
    pub temp: f64,
}

#[derive(Serialize, Deserialize)]
pub struct Daily {
    pub temp: DailyTemp,
}

#[derive(Serialize, Deserialize)]
pub struct DailyTemp {
    pub morn: f64,
    pub day: f64,
    pub eve: f64,
    pub night: f64,
    pub min: f64,
    pub max: f64,
}
