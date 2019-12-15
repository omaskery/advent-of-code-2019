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

    fn get(&self, name: &str) -> Option<&Body> {
        self.bodies.get(name)
    }

    fn get_parent_of(&self, name: &str) -> Option<&Body> {
        let body = self.get(name)?;
        self.get(&body.parent.as_ref()?)
    }

    fn get_path_to(&self, name: &str) -> Option<Vec<&Body>> {
        let mut body = self.get(name)?;
        let mut path = Vec::new();
        while let Some(parent) = self.get_parent_of(&body.name) {
            body = parent;
            path.push(body);
        }
        Some(path)
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
            .ok_or(OrbitError::InvalidBodyName)
            .compat()?;
        total_orbits += path_to_body.len();
    }

    println!("total orbits: {}", total_orbits);

    Ok(())
}