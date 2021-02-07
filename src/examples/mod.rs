use std::collections::HashMap;
use std::fs;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
pub struct Example {
    request: Option<Value>,
    response: Option<Value>,
}

#[derive(Serialize, Debug)]
pub struct Bag {
    pub examples: HashMap<String, Example>,
}

impl Clone for Example {
    fn clone(&self) -> Self {
        Example {
            request: self.request.clone(),
            response: self.response.clone(),
        }
    }
}

fn get_file_content(file_name: &str) -> Option<String> {
    match fs::read_to_string(file_name) {
        Ok(content) => Some(content),
        _ => None,
    }
}

impl Bag {
    pub fn new_from_file(file_name: &str) -> Bag {
        let mut bag = Bag {
            examples: HashMap::new(),
        };
        if let Some(content) = get_file_content(file_name) {
            if let Ok(json_example) = serde_json::from_str(content.as_str()) {
                bag.examples = json_example;
            }
        }
        bag
    }
}
