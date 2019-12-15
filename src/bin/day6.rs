use std::error::Error;
use std::collections::HashMap;
use std::str::FromStr;
use advent_of_code_2019::load_lines_from;
use failure::{Fail, ResultExt};

#[derive(Copy, Clone, Debug, Fail, Eq, PartialEq)]
enum OrbitError {
    #[fail(display = "failed to parse orbit data")]
    ParseError,
    #[fail(display = "attempted to lookup a body with an invalid name")]
    InvalidBodyName,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Body {
    parent: Option<String>,
    name: String,
}

impl FromStr for Body {
    type Err = OrbitError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.splitn(2, ")")
            .map(|string| string.into())
            .collect::<Vec<String>>();

        if parts.len() != 2 {
            return Err(OrbitError::ParseError);
        }

        Ok(Body {
            parent: Some(parts[0].clone()),
            name: parts[1].clone(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct OrbitData {
    bodies: HashMap<String, Body>,
}

impl OrbitData {
    fn new(bodies: &[Body]) -> OrbitData {
        let mut result = OrbitData {
            bodies: bodies.iter()
                .cloned()
                .map(|body| (body.name.clone(), body))
                .collect(),
        };
        result.bodies.insert("COM".into(), Body {
            name: "COM".into(),
            parent: None,
        });
        result
    }

    fn get(&self, name: &str) -> Result<&Body, OrbitError> {
        Ok(self.bodies.get(name).ok_or(OrbitError::InvalidBodyName)?)
    }

    fn get_parent_of(&self, name: &str) -> Result<Option<&Body>, OrbitError> {
        let body = self.get(name)?;
        if let Some(parent_name) = &body.parent {
            Ok(Some(self.get(&parent_name)?))
        } else {
            Ok(None)
        }
    }

    fn get_path_to(&self, name: &str) -> Result<Vec<&Body>, OrbitError> {
        let mut body = self.get(name)?;
        let mut path = Vec::new();
        while let Some(parent) = self.get_parent_of(&body.name)? {
            body = parent;
            path.push(body);
        }
        path.reverse();
        Ok(path)
    }

    fn get_name_path_to(&self, name: &str) -> Result<Vec<String>, OrbitError> {
        Ok(self.get_path_to(name)?
            .iter()
            .map(|body| body.name.clone())
            .collect::<Vec<_>>())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let orbit_data = OrbitData::new(&load_lines_from("input/day6.txt")?
        .into_iter()
        .map(|line| line.parse::<Body>())
        .collect::<Result<Vec<_>, _>>()
        .compat()?);

    let mut total_orbits = 0;
    for body in orbit_data.bodies.keys() {
        let path_to_body = orbit_data.get_path_to(body)
            .compat()?;
        total_orbits += path_to_body.len();
    }

    let body_i_orbit = orbit_data.get_parent_of("YOU").compat()?
        .expect("I'm not in the map data?").name.clone();
    let body_santa_orbits = orbit_data.get_parent_of("SAN").compat()?
        .expect("Santa isn't in the map data?").name.clone();

    let path_to_my_body = orbit_data.get_name_path_to(&body_i_orbit).compat()?;
    let path_to_santas_body = orbit_data.get_name_path_to(&body_santa_orbits).compat()?;

    println!("path to me: {:?}", path_to_my_body);
    println!("path to santa: {:?}", path_to_santas_body);

    let common_ancestor = path_to_my_body.iter().zip(path_to_santas_body.iter())
        .take_while(|(a, b)| *a == *b)
        .last()
        .map(|(a, _)| a)
        .expect("no common ancestor for calculating transfer path");

    let index_into_path = path_to_my_body.iter()
        .enumerate()
        .find(|(_, value)| *value == common_ancestor)
        .map(|(index, _)| index)
        .unwrap();

    let transfers_to_me = path_to_my_body.len() - index_into_path;
    let transfers_to_santa = path_to_santas_body.len() - index_into_path;
    let total_transfers = transfers_to_me + transfers_to_santa;

    println!("total orbits: {}", total_orbits);
    println!("both I and Santa (in)directly orbit {}", common_ancestor);
    println!("  transfers to me: {}", transfers_to_me);
    println!("  transfers to santa: {}", transfers_to_santa);
    println!("  total transfers: {}", total_transfers);

    Ok(())
}