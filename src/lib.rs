use wasm_bindgen::prelude::*;
use crate::models::{Entity, Enum, ProjectModel};
use std::collections::HashMap;
use serde::Serialize;

mod models;

#[derive(Serialize)]
pub struct ModelsProject {
    pub enums: Vec<Enum>,
    pub entities: Vec<ModelEntity>,
}

#[derive(Serialize)]
pub struct ModelEntityField {
    pub name: String,
    pub data_type: String,
    pub description: String,
    /// use to represent arrays or maps
    pub is_array: bool,
    pub example: String,
    pub markers: Vec<String>,
    pub tags: HashMap<String, String>,
    /// used only for enums
    pub allowed_values: Vec<String>,
    pub is_required: bool,
}

#[derive(Serialize)]
pub struct ModelEntity {
    pub name: String,
    pub fields: Vec<ModelEntityField>,
}

fn to_summarized_entity(from: Entity) -> ModelEntity {
    let mut new_fields = Vec::new();
    for field in from.fields.into_iter() {
        let mut is_required = false;
        for v in &field.1.markers {
            if v == "required" {
                is_required = true;
                break;
            }
        }
        let mut example = field.1.example;
        if example.starts_with("\"") && example.ends_with("\""){
            example.remove(example.len() - 1);
            example.remove(0);
        }
        let f = ModelEntityField {
            name: field.0,
            data_type: field.1.data_type,
            description: field.1.description,
            is_array: field.1.is_array,
            example: "".to_string(),
            markers: field.1.markers,
            tags: field.1.tags,
            allowed_values: field.1.allowed_values,
            is_required
        };
        new_fields.push(f);
    }
    ModelEntity {
        name: from.name,
        fields: new_fields,
    }
}

fn convert(from: ProjectModel) -> ModelsProject {
    let mut enums = Vec::new();
    let mut entities = Vec::new();
    for en in from.enums.into_iter() {
        enums.push(en.1);
    }
    for entity_pair in from.entities.into_iter() {
        entities.push(to_summarized_entity(entity_pair.1));
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