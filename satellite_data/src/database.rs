use crate::{data::{astronomical_data::{OrbitalParams, PhysicalParams}}, satellites::Satellite};
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::{Read, Write};
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct  Database {
    pub data: Vec<Satellite>
}

impl PartialEq for Database {
    fn eq(&self, other: &Self) -> bool {
        self.data.len() == other.data.len()
    }
}

impl Eq for Database {}

impl Database {
    pub fn from_json(file: &mut File) -> Database {
        let mut file_content = String::new();
        file.read_to_string(&mut file_content).unwrap();
        let deserialized: Database = serde_json::from_str(&file_content).unwrap();
        deserialized
    }
    pub fn to_json(&self, file: &mut File) {
        let serialized = serde_json::to_string(self).unwrap();
        file.write(&serialized.as_bytes()).unwrap();
    }
    pub fn parse_data(physical_data: &mut File, orbital_data: &mut File) -> Database {
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
            println!("{}, {}", orbital_data_vars[1], orbital_data_vars.len());
            let orbital_data_vars: Vec<&str> = orbital_data_vars[0..12].iter().map(|content| {
                *content
            }).collect();


            
            let physical_data_vars: Vec<&str> = physical_data_lines.iter().find(|line| line.contains(orbital_data_vars[1]) ).unwrap().split("	").collect();

            let physical_data_vars: Vec<&str> = physical_data_vars.iter().filter(|content| {
                !content.contains("[") && content != &&" "
            }).map(|content| {
                *content
            }).collect(); 

            let physical_params = PhysicalParams {
                gm: physical_data_vars[1].parse().unwrap(),
                radius: physical_data_vars[2].parse().unwrap(),
                density: physical_data_vars[3].parse().unwrap(),
                magnitude: physical_data_vars[4].parse().unwrap(),
                albedo: physical_data_vars[5].parse().unwrap()
            };
            let orbital_params = OrbitalParams {
                major_semiaxis: orbital_data_vars[2].parse().unwrap(),
                eccentricity: orbital_data_vars[3].parse().unwrap(),
                inclination: orbital_data_vars[6].parse().unwrap(),
                ascending_node: orbital_data_vars[7].parse().unwrap() 
            };
            let satellite = Satellite {
                major_body: orbital_data_vars[0].parse().unwrap(),
                name: orbital_data_vars[1].to_string(),
                id: index,
                orbital_params,
                physical_params
            };
            data.push(satellite);
        }
        Database {data}
    }
    pub fn get_satellite_by_id(&self, id: usize) -> Option<Satellite> {
        let satellites: Vec<Satellite> = self.data.iter().filter(|&value| value.id == id).map(|satellite| satellite.clone()).collect();
        match satellites.first() {
            Some(satellite) => Some(satellite.clone()),
            None => None
        }
    }
}