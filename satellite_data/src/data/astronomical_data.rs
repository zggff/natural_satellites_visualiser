use crate::data::data_with_error::DataWithError;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct OrbitalParams{
    pub major_semiaxis: f64,
    pub eccentricity: f64,
    pub inclination: f64,
    pub ascending_node: f64,
}
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct PhysicalParams {
    pub gm: DataWithError<f64>,
    pub radius: DataWithError<f64>,
    pub density: DataWithError<f64>,
    pub magnitude: DataWithError<f64>,
    pub albedo: DataWithError<f64>
}

impl PhysicalParams {
    pub fn new(
                gm: DataWithError<f64>,
                radius: DataWithError<f64>,
                density: DataWithError<f64>,
                magnitude: DataWithError<f64>,
                albedo: DataWithError<f64>
                ) -> PhysicalParams {
        PhysicalParams {gm, radius, density, magnitude, albedo}
    }
}

