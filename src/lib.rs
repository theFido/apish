use wasm_bindgen::prelude::*;
use crate::models::{Entity, Enum, ProjectModel};
use serde::Serialize;

mod models;

#[derive(Serialize)]
pub struct ModelsProject {
    pub enums: Vec<Enum>,
    pub entities: Vec<Entity>,
}

fn convert(from: ProjectModel) -> ModelsProject {
    let mut enums = Vec::new();
    let mut entities = Vec::new();
    for en in from.enums.into_iter() {
        enums.push(en.1);
    }
    for entityPair in from.entities.into_iter() {
        entities.push(entityPair.1);
    }
    ModelsProject {
        enums,
        entities,
    }
}

#[wasm_bindgen]
pub fn parse_models(from: &str) -> String {
    let res = models::get_models(from);
    let project = convert(res);
    serde_json::to_string(&project).unwrap()
}