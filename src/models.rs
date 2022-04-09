use std::collections::HashMap;
use std::hash::Hash;

use pest::Parser;
use pest::iterators::Pair;

use serde::{Serialize};

#[derive(Parser)]
#[grammar = "grammar_models.pest"]
struct ModelsParser;

#[derive(Debug, Serialize)]
pub struct Field {
    pub identifier: String,
    pub data_type: String,
    pub description: String,
    /// use to represent arrays or maps
    pub is_array: bool,
    pub example: String,
    pub markers: Vec<String>,
    pub tags: HashMap<String, String>,
    /// used only for enums
    pub allowed_values: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct Entity {
    pub name: String,
    pub fields: HashMap<String, Field>,
}

#[derive(Debug, Serialize)]
struct Optionals {
    markers: Vec<String>,
    tags: HashMap<String, String>,
}

#[derive(Debug, Serialize)]
pub struct Enum {
    pub name: String,
    pub values: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ProjectModel {
    pub entities: HashMap<String, Entity>,
    pub enums: HashMap<String, Enum>,
}

impl Clone for ProjectModel {
    fn clone(&self) -> Self {
        ProjectModel {
            entities: self.entities.clone(),
            enums: self.enums.clone()
        }
    }
}

impl Clone for Field {
    fn clone(&self) -> Self {
        Field {
            identifier: self.identifier.to_string(),
            data_type: self.data_type.to_string(),
            description: self.description.to_string(),
            is_array: self.is_array,
            example: self.example.to_string(),
            markers: self.markers.clone(),
            tags: self.tags.clone(),
            allowed_values: self.allowed_values.clone(),
        }
    }
}

impl Clone for Entity {
    fn clone(&self) -> Self {
        Entity{
            name: self.name.to_string(),
            fields: self.fields.clone(),
        }
    }
}

impl Clone for Enum {
    fn clone(&self) -> Self {
        Enum {
            name: self.name.to_string(),
            values: self.values.clone(),
        }
    }
}

fn get_enum_field(pair: Pair<Rule>) -> String {
    for arg_pair in pair.into_inner() {
        match arg_pair.as_rule() {
            Rule::enumInnerItem => {
                return arg_pair.into_inner().next().unwrap().as_str().to_string();
            },
            Rule::ident => {
                return arg_pair.as_str().to_string();
            }
            _ => {
                return "".to_string();
            }
        }
    }
    return "".to_string()
}

fn get_enum_definition(pair: Pair<Rule>) -> Vec<String> {
    let mut values: Vec<String> = Vec::new();
    for arg_pair in pair.into_inner() {
        match arg_pair.as_rule() {
            Rule::enumItem => {
                values.push(get_enum_field(arg_pair));
            },
            _ => {
                // nothing to do
            }
        }
    }
    values
}

fn get_enum(pair: Pair<Rule>) -> Enum {
    let mut name: String = "".to_string();
    let mut values = Vec::new();
    for arg_pair in pair.into_inner() {
        match arg_pair.as_rule() {
            Rule::enumName => {
                name =  arg_pair.as_str().to_string();
            },
            Rule::enumDef => {
                values = get_enum_definition(arg_pair);
            }
            _ => {
                // nothing to do
            }
        }
    }
    Enum {
        name,
        values,
    }
}

fn get_identifiers(pair: Pair<Rule>) -> Vec<String> {
    let mut tokens = Vec::new();
    for arg_pair in pair.into_inner() {
        match arg_pair.as_rule() {
            Rule::ident => {
                tokens.push(arg_pair.as_str().to_string());
            }
            _ => {
                println!("another brick in the wall");
            }
        }
    }
    tokens
}

fn get_object_optionals(pair: Pair<Rule>) -> Optionals {
    let mut markers = Vec::new();
    let mut tags = HashMap::new();
    let mut last_key = "".to_string();
    for arg_pair in pair.into_inner() {
        match arg_pair.as_rule() {
            Rule::objMarkers => {
                markers = get_identifiers(arg_pair);
            }
            Rule::objTags => {
                for inner in arg_pair.into_inner() {
                    match inner.as_rule() {
                        Rule::fieldName => {
                            last_key = inner.as_str().to_string();
                        }
                        Rule::ident => {
                            let val = inner.as_str().to_string();
                            tags.insert(last_key.to_owned(), val);
                        }
                        _ => {
                            println!("tags");
                        }
                    }
                }
            }
            _ => {
                // skip
            }
        }
    }
    Optionals {
        markers,
        tags,
    }
}

fn get_object_field(pair: Pair<Rule>) -> Field {
    let mut identifier: String = "".to_string();
    let mut data_type: String = "".to_string();
    let mut is_array = false;
    let mut description = "".to_string();
    let mut markers = Vec::new();
    let mut tags = HashMap::new();
    for arg_pair in pair.into_inner() {
        match arg_pair.as_rule() {
            Rule::objOptionals => {
                let opts = get_object_optionals(arg_pair);
                if opts.markers.len() > 0 {
                    markers = opts.markers;
                }
                if opts.tags.len() > 0 {
                    for (k, v) in opts.tags {
                        tags.insert(k, v);
                    }
                }
            }
            Rule::objDescription => {
                description = arg_pair.as_str().to_owned();
            }
            Rule::fieldType => {
                data_type = arg_pair.as_str().to_owned();
            }
            Rule::fieldName => {
                identifier = arg_pair.as_str().to_owned();
            }
            Rule::arrayIndicator => {
                is_array = true;
            }
            // Rule::
            _ => {

                // only white space + new line here, ignore
            }
        }
    }
    let mut example = "".to_string();
    if tags.contains_key("example") {
        example = tags.get("example").unwrap().to_owned();
    }
    let default_field = Field {
        identifier,
        data_type,
        description,
        is_array,
        example,
        markers,
        tags,
        allowed_values: vec![]
    };
    default_field
}

fn get_entity_field(pair: Pair<Rule>) -> Field {
    for arg_pair in pair.into_inner() {
        match arg_pair.as_rule() {
            Rule::objField => {
                return get_object_field(arg_pair);
            }
            _ => {
                println!("- '{}'", arg_pair.as_str());
            }
        }
    }
    Field {
        identifier: "".to_string(),
        data_type: "".to_string(),
        description: "".to_string(),
        is_array: false,
        example: "".to_string(),
        markers: vec![],
        tags: Default::default(),
        allowed_values: vec![]
    }
}

fn get_entity(pair: Pair<Rule>) -> Entity {
    let mut name: String = "".to_string();
    let mut fields = HashMap::new();
    for arg_pair in pair.into_inner() {
        match arg_pair.as_rule() {
            Rule::ident => {
                name = arg_pair.as_str().to_string();
            },
            Rule::objDef => {
                for arg_pair_alt in arg_pair.into_inner() {
                    let field = get_entity_field(arg_pair_alt);
                    fields.insert(field.identifier.to_owned(), field);
                }
            },
            _ => {
                // nothing to do
            }
        }
    }
    Entity {
        name,
        fields,
    }
}

pub fn get_models(from_model: &str) -> ProjectModel {
    let content = ModelsParser::parse(Rule::definitions, from_model)
        .expect("cannot parse")
        .next().unwrap();

    let mut enums = HashMap::new();
    let mut entities = HashMap::new();
    for record in content.into_inner() {
        match record.as_rule() {
            Rule::enu => {
                let val = get_enum(record);
                enums.insert(val.name.to_owned(), val);
            }
            Rule::objType => {
                let val = get_entity(record);
                entities.insert(val.name.to_owned(), val);
            }
            _ => {
                // println!("other");
            }
        }
    }
    ProjectModel {
        entities,
        enums,
    }
}

#[test]
fn test_parser() {
    let input = include_str!("models_def.model");
    let result = get_models(input);
    let mood = result.enums.get("Mood").unwrap();

    assert_eq!(mood.name, "Mood");
    assert_eq!(mood.values, vec!["happy", "mad", "sad"]);

    let person = result.entities.get("Person").unwrap();
    assert_eq!(person.name, "Person");
    assert_eq!(person.fields.len(), 4);
    assert_eq!(person.fields.get("name").unwrap().markers.len(), 1);
    assert_eq!(person.fields.get("name").unwrap().markers.get(0).unwrap(), "required");
    assert_eq!(person.fields.get("name").unwrap().example, "123");

    let class_room = result.entities.get("ClassRoom").unwrap();
    assert_eq!(class_room.name, "ClassRoom");
}