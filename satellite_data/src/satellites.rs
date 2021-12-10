use std::str::FromStr;

use super::data::astronomical_data::{PhysicalParams, OrbitalParams};
use serde::{Serialize, Deserialize};
use std::error::Error;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Satellite {
    pub major_body: MajorBody,
    pub name: String,
    pub id: usize,
    pub orbital_params: OrbitalParams,
    pub physical_params: PhysicalParams
}

impl PartialEq for Satellite {
    fn eq(&self, other: &Satellite) -> bool {
        self.name == other.name && self.orbital_params.major_semiaxis == other.orbital_params.major_semiaxis
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
pub enum MajorBody {
    Earth,
    Mars,
    Jupiter,
    Saturn,
    Uranus,
    Neptune,
    Pluto
}

impl FromStr for MajorBody {
    type Err = UnknownPlanetError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        let s = s.as_str();
        match s {
            "earth" => Ok(MajorBody::Earth),
            "mars" => Ok(MajorBody::Mars),
            "jupiter" => Ok(MajorBody::Jupiter),
            "saturn" => Ok(MajorBody::Saturn),
            "uranus" => Ok(MajorBody::Uranus),
            "neptune" => Ok(MajorBody::Neptune),
            "pluto" => Ok(MajorBody::Pluto),
            _ => Err(UnknownPlanetError)
        }
    }
}

impl ToString for MajorBody {
    fn to_string(&self) -> String {
        match self {
            MajorBody::Earth => String::from("Earth"),
            MajorBody::Mars => String::from("Mars"),
            MajorBody::Jupiter => String::from("Jupiter"),
            MajorBody::Saturn => String::from("Saturn"),
            MajorBody::Uranus => String::from("Uranus"),
            MajorBody::Neptune => String::from("Neptune"),
            MajorBody::Pluto => String::from("Pluto"),
        }
    }
}

#[derive(Debug)]
pub struct UnknownPlanetError;

impl std::fmt::Display for UnknownPlanetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unknown planet")
    }
}

impl Error for UnknownPlanetError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}