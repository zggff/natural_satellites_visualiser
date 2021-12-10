use crate::{
    data::astronomical_data::{OrbitalParams, PhysicalParams},
    satellites::Satellite,
};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::io::{Read, Write};
use std::{fs::File, path::Path};
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Database {
    pub data: Vec<Satellite>,
}

impl PartialEq for Database {
    fn eq(&self, other: &Self) -> bool {
        self.data.len() == other.data.len()
    }
}

impl Eq for Database {}

impl Database {
    pub fn from_json(file: impl AsRef<Path>) -> Result<Database, Box<dyn Error>> {
        let mut file = File::open(file)?;
        let mut file_content = String::new();
        file.read_to_string(&mut file_content)?;
        let deserialized: Database = serde_json::from_str(&file_content)?;
        Ok(deserialized)
    }
    pub fn to_json(&self, file: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
        let serialized = serde_json::to_string(self).unwrap();
        if std::fs::metadata(&file).is_ok() {
            panic!("file already exists");
        }
        let mut file = File::create(file)?;
        file.write(&serialized.as_bytes())?;
        Ok(())
    }
    pub fn from_raw_data(
        orbital_data: impl AsRef<Path>,
        physical_data: impl AsRef<Path>,
    ) -> Result<Database, Box<dyn Error>> {
        let mut physical_data = File::open(physical_data)?;
        let mut orbital_data = File::open(orbital_data)?;
        let mut file_content = String::new();
        physical_data.read_to_string(&mut file_content).unwrap();
        let physical_data_lines = file_content.lines();
        let physical_data_lines: Vec<&str> = physical_data_lines.collect();

        let mut file_content = String::new();
        orbital_data.read_to_string(&mut file_content).unwrap();
        let orbital_data_lines = file_content.lines();

        let mut data: Vec<Satellite> = Vec::new();
        for (index, line) in orbital_data_lines.enumerate() {
            if index == 0 {
                continue;
            }
            let orbital_data_vars: Vec<&str> = line.split("	").collect();
            let orbital_data_vars: Vec<&str> = orbital_data_vars[0..12]
                .iter()
                .map(|content| *content)
                .collect();

            let physical_data_vars = physical_data_lines
                .iter()
                .find(|line| line.contains(orbital_data_vars[1]));

            let physical_data_vars: Vec<&str> = match physical_data_vars {
                Some(vars) => vars,
                None => break,
            }
            .split("	")
            .collect();

            let physical_data_vars: Vec<&str> = physical_data_vars
                .iter()
                .filter(|content| !content.contains("[") && content != &&" ")
                .map(|content| *content)
                .collect();

            let physical_params = PhysicalParams {
                gm: physical_data_vars[1].parse()?,
                radius: physical_data_vars[2].parse()?,
                density: physical_data_vars[3].parse()?,
                magnitude: physical_data_vars[4].parse()?,
                albedo: physical_data_vars[5].parse()?,
            };
            let orbital_params = OrbitalParams {
                major_semiaxis: orbital_data_vars[2].parse()?,
                eccentricity: orbital_data_vars[3].parse()?,
                inclination: orbital_data_vars[6].parse()?,
                ascending_node: orbital_data_vars[7].parse()?,
            };
            let satellite = Satellite {
                major_body: orbital_data_vars[0].parse()?,
                name: orbital_data_vars[1].to_string(),
                id: index,
                orbital_params,
                physical_params,
            };
            data.push(satellite);
        }
        Ok(Database { data })
    }
    pub fn get_satellite_by_id(&self, id: usize) -> Option<Satellite> {
        let satellites: Vec<Satellite> = self
            .data
            .iter()
            .filter(|&value| value.id == id)
            .map(|satellite| satellite.clone())
            .collect();
        match satellites.first() {
            Some(satellite) => Some(satellite.clone()),
            None => None,
        }
    }
}
