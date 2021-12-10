use std::{num::ParseFloatError, str::FromStr};

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct DataWithError<T>{
    pub data: T,
    pub error: T,
}

impl <T> DataWithError <T> where T:Clone{
    pub fn to_value(&self) -> T {
        self.data.clone()
    }
}

impl From<(f64, f64)> for DataWithError<f64> {
    fn from(data: (f64,f64)) -> DataWithError<f64>{
        DataWithError{data: data.0, error: data.1}
    }
}

impl From<f64> for DataWithError<f64> {
    fn from(data: f64) -> DataWithError<f64>{
        DataWithError{data, error: 0.0}
    }
}

impl From<DataWithError<f64>> for f64{
    fn from(data: DataWithError<f64>) -> f64 {
        data.data
    }
}

impl From<DataWithError<f64>> for (f64, f64){
    fn from(data: DataWithError<f64>) -> (f64, f64) {
        (data.data, data.error)
    }
}

impl FromStr for DataWithError<f64> {
    type Err = ParseFloatError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains("±") {
            let parts: Vec<&str> = s.split("±").collect();
            if parts[0].contains("R") || parts[0].contains("r") || parts[0].contains("v") || parts[0].contains("V") {
                let s = &parts[0][0..parts[0].len()-1];
                let data: f64 = s.parse()?;
                let error: f64 = parts[1].parse()?;
                return Ok (DataWithError {data, error})            
            }
            let data: f64 = parts[0].parse()?;
            let error: f64 = parts[1].parse()?;
            Ok (DataWithError {data, error})            
        } else if s.contains("R") || s.contains("r") || s.contains("v") || s.contains("V"){
            let len = s.len()-1;
            let s = &s[0..len];
            let data: f64 = s.parse()?;
            Ok (DataWithError {data, error: 0.0})
        } else if s.contains("?") {
            Ok (DataWithError {data: 0.0, error: -1.0})
        } else  {
            let data: f64 = s.parse()?;
            Ok (DataWithError {data, error: 0.0})
        }
    }
}

impl ToString for DataWithError<f64> {
    fn to_string(&self) -> String {
        let mut string = self.data.to_string();
        if self.error < 0.0 {
            string = "?".to_string()
        } else if self.error > 0.0 {
            string.push_str("±");
            string.push_str(&self.error.to_string())
        }
        string
    }
}