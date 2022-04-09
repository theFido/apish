use crate::examples;

use std::collections::HashMap;
use std::fs;

use examples::Bag;

use pest::iterators::Pair;
use pest::Parser;
use serde::Serialize;
use crate::models::{get_models, ProjectModel};

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct ApishParser;

// ToDo: Replace me
#[derive(Debug)]
struct APIWrapper {
    endpoint: String,
    definition: APIDefinition,
}

#[derive(Debug)]
enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Unknown,
}

enum PairModifiers {
    Alias,
    Unknown,
}

#[derive(Debug, Serialize)]
pub struct APIDefinition {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub get: Option<APIConfiguration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post: Option<APIConfiguration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub put: Option<APIConfiguration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete: Option<APIConfiguration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch: Option<APIConfiguration>,
}

#[derive(Debug, Serialize)]
pub enum DataType {
    String,
    Number,
    Boolean,
    Unknown,
}

#[derive(Debug, Serialize)]
pub struct StatusCode {
    pub code: String,
    pub description: String,
    pub is_retryable: bool,
}

#[derive(Debug, Serialize)]
pub struct ProjectArgument {
    pub name: String,
    pub data_type: DataType,
    pub alias: String,
    pub required: bool,
    pub default_value: String,
    pub description: String,
}

#[derive(Debug, Serialize)]
struct ArgumentGroup {
    id: String,
    items: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct Project {
    pub title: String,
    pub version: String,
    headers_groups: Vec<ArgumentGroup>,
    params_groups: Vec<ArgumentGroup>,
    query_groups: Vec<ArgumentGroup>,
    status_codes_groups: Vec<ArgumentGroup>,
    pub headers: Vec<ProjectArgument>,
    pub query: Vec<ProjectArgument>,
    pub params: Vec<ProjectArgument>,
    pub status_codes: Vec<StatusCode>,
    pub endpoints: HashMap<String, APIDefinition>,
    pub examples: HashMap<String, Vec<examples::Example>>,
    pub models: Option<ProjectModel>,
}

#[derive(Debug, Serialize)]
pub struct APIConfiguration {
    pub description: String,
    pub operation: String,
    pub query_string: Vec<String>,
    pub path_params: Vec<String>,
    pub headers: Vec<String>,
    pub tags: Vec<String>,
    pub status_codes: Vec<String>,
    pub produces: Vec<String>,
    pub consumes: Vec<String>,
    pub use_cases: Vec<String>,
    pub example: String,
}

impl Clone for DataType {
    fn clone(&self) -> Self {
        match self {
            DataType::String => DataType::String,
            DataType::Boolean => DataType::Boolean,
            DataType::Number => DataType::Number,
            _ => DataType::Unknown,
        }
    }
}

impl DataType {
    pub fn as_str(&self) -> &'static str {
        match *self {
            DataType::String => "string",
            DataType::Number => "number",
            DataType::Boolean => "boolean",
            DataType::Unknown => "unk",
        }
    }
}

impl PartialEq for DataType {
    fn eq(&self, other: &Self) -> bool {
        if self.as_str() == other.as_str() {
            return true;
        }
        false
    }
}

impl StatusCode {
    fn new(code: &str, description: &str) -> StatusCode {
        StatusCode {
            code: code.to_string(),
            description: description.to_string(),
            is_retryable: false,
        }
    }
}

impl PartialEq for ProjectArgument {
    fn eq(&self, other: &Self) -> bool {
        if other.name != self.name
            || other.required != self.required
            || other.default_value != self.default_value
            || other.data_type != self.data_type
            || other.alias != self.alias
        {
            return false;
        }
        true
    }
}

impl Project {
    fn new(examples: HashMap<String, Vec<examples::Example>>, models: Option<ProjectModel>) -> Project {
        Project {
            title: "".to_string(),
            version: "".to_string(),
            headers_groups: vec![],
            params_groups: vec![],
            query_groups: vec![],
            status_codes_groups: vec![],
            headers: vec![],
            query: vec![],
            params: vec![],
            status_codes: vec![],
            endpoints: HashMap::new(),
            examples,
            models,
        }
    }

    pub fn new_from_file(file_name: String, models_file: String, examples_file: String) -> Result<Project, String> {
        let api_definition = get_file_content(file_name);
        if let Err(e) = api_definition {
            return Err(e);
        }
        let mut models: Option<ProjectModel> = None;
        match get_file_content(models_file) {
            Ok(file_content) => {
                models = Some(get_models(file_content.as_ref()));
            }
            _ => {
                // do nothing
            }
        }
        let bag = Bag::new_from_file(examples_file.as_str());
        match ApishParser::parse(Rule::api_file, &api_definition.unwrap().as_ref()) {
            Ok(mut pairs) => {
                let n = pairs.next();
                match n {
                    Some(pair) => {
                        let mut project = Project::new(bag.examples, models);

                        parse_value(pair, &mut project);
                        Ok(project)
                    }
                    None => Err("Cannot process file".to_string()),
                }
            }
            Err(e) => Err(format!("Cannot parse API definition {}", e)),
        }
    }

    fn get_header(&self, name: &str) -> Option<&ProjectArgument> {
        for header in &self.headers {
            if header.alias == name || header.name == name {
                return Some(header);
            }
        }
        None
    }

    fn get_query_string(&self, name: &str) -> Option<&ProjectArgument> {
        for query in &self.query {
            if query.alias == name || query.name == name {
                return Some(query);
            }
        }
        None
    }

    fn get_path_param(&self, name: &str) -> Option<&ProjectArgument> {
        for path_param in &self.params {
            if path_param.alias == name || path_param.name == name {
                return Some(path_param);
            }
        }
        None
    }

    fn get_status_code(&self, status_code: &str) -> Option<StatusCode> {
        for code in &self.status_codes {
            if code.code == status_code {
                let project_status = StatusCode {
                    code: code.code.to_string(),
                    description: code.description.to_string(),
                    is_retryable: code.is_retryable,
                };
                return Some(project_status);
            }
        }
        match status_code {
            "200" => Some(StatusCode::new(status_code, "Ok")),
            "201" => Some(StatusCode::new(status_code, "Created")),
            "202" => Some(StatusCode::new(status_code, "Accepted")),
            "204" => Some(StatusCode::new(status_code, "No Content")),
            "400" => Some(StatusCode::new(status_code, "Bad Request")),
            "401" => Some(StatusCode::new(status_code, "Unauthorized")),
            "403" => Some(StatusCode::new(status_code, "Forbidden")),
            "404" => Some(StatusCode::new(status_code, "Not Found")),
            "405" => Some(StatusCode::new(status_code, "Method Not Allowed")),
            "406" => Some(StatusCode::new(status_code, "Not Acceptable")),
            "408" => Some(StatusCode::new(status_code, "Request Timeout")),
            "413" => Some(StatusCode::new(status_code, "Payload Too Large")),
            "415" => Some(StatusCode::new(status_code, "Unsupported Media Type")),
            "417" => Some(StatusCode::new(status_code, "Expectation Failed")),
            "418" => Some(StatusCode::new(status_code, "I'm a teapot")),
            "424" => Some(StatusCode::new(status_code, "Failed Dependency")),
            "429" => Some(StatusCode::new(status_code, "Too Many Requests")),
            "500" => Some(StatusCode::new(status_code, "Internal Server Error")),
            "501" => Some(StatusCode::new(status_code, "Not Implemented")),
            "502" => Some(StatusCode::new(status_code, "Bad Gateway")),
            "503" => Some(StatusCode::new(status_code, "Service Unavailable")),
            "504" => Some(StatusCode::new(status_code, "Gateway Timeout")),
            _ => None,
        }
    }

    /// Returns the expanded headers associated to a string of tokens
    pub fn get_headers(&self, list: &[String]) -> Vec<&ProjectArgument> {
        let mut headers = Vec::new();
        for item in list {
            if let Some(header) = self.get_header(item) {
                headers.push(header);
            }
        }
        headers
    }

    /// Returns the expanded query string associated to a string of tokens
    pub fn get_query_strings(&self, list: &[String]) -> Vec<&ProjectArgument> {
        let mut query_strings = Vec::new();
        for item in list {
            if let Some(qs) = self.get_query_string(item) {
                query_strings.push(qs);
            }
        }
        query_strings
    }

    /// Returns the expanded path parameters associated to a string of tokens
    pub fn get_path_params(&self, list: &[String]) -> Vec<&ProjectArgument> {
        let mut path_params = Vec::new();
        for item in list {
            if let Some(path_param) = self.get_path_param(item) {
                path_params.push(path_param);
            }
        }
        path_params
    }

    /// Returns the list of status code descriptions
    pub fn get_status_codes(&self, list: &[String]) -> Vec<StatusCode> {
        let mut codes = Vec::new();
        for status in list {
            if let Some(sc) = self.get_status_code(status) {
                codes.push(sc);
            }
        }
        codes
    }

    /// Return the individual items referenced by a group ID
    fn spread_group(&self, section: &str, group_name: &str) -> Vec<String> {
        let mut items = Vec::new();
        let optional_source = match section {
            "headers" => Some(&self.headers_groups),
            "params" => Some(&self.params_groups),
            "query" => Some(&self.query_groups),
            "status_codes" => Some(&self.status_codes_groups),
            _ => {
                // ignore
                println!("{}", section);
                None
            }
        };
        if let Some(source) = optional_source {
            for item in source {
                if item.id == group_name {
                    for alias in &item.items {
                        items.push(alias.to_owned());
                    }
                }
            }
        }
        items
    }
}

impl ProjectArgument {
    #[cfg(test)]
    fn new(
        name: &str,
        data_type: DataType,
        alias: &str,
        required: bool,
        default_value: &str,
    ) -> ProjectArgument {
        ProjectArgument {
            name: name.to_string(),
            data_type,
            alias: alias.to_string(),
            required,
            default_value: default_value.to_string(),
            description: "".to_string(),
        }
    }
}

fn parse_argument(pair: Pair<Rule>) -> ProjectArgument {
    let mut arg = ProjectArgument {
        name: "".to_string(),
        description: "".to_string(),
        data_type: DataType::String,
        alias: "".to_string(),
        required: false,
        default_value: "".to_string(),
    };
    for arg_pair in pair.into_inner() {
        match arg_pair.as_rule() {
            Rule::ident => {
                arg.name = arg_pair.as_str().to_owned();
            }
            Rule::options => {
                for opt in arg_pair.into_inner() {
                    match opt.as_rule() {
                        Rule::modified_pair => {
                            let mut key = PairModifiers::Unknown;
                            let mut value = String::new();
                            for k in opt.into_inner() {
                                match k.as_rule() {
                                    Rule::pair_modifiers => {
                                        if k.as_str() == "alias" {
                                            key = PairModifiers::Alias;
                                        }
                                    }
                                    Rule::ident => {
                                        value = k.as_str().to_owned();
                                    }
                                    _ => {
                                        // println!("skipped mod_pair {}", k);
                                    }
                                }
                            }
                            if let PairModifiers::Alias = key {
                                arg.alias = value;
                            }
                        }
                        Rule::single_modifiers => {
                            if opt.as_str() == "required" {
                                arg.required = true;
                            }
                        }
                        Rule::default_value => {
                            for dv_inner in opt.into_inner() {
                                if let Rule::ident = dv_inner.as_rule() {
                                    arg.default_value = dv_inner.as_str().to_owned();
                                }
                            }
                        }
                        _ => {
                            // println!("skipped option {}", opt);
                        }
                    }
                }
            }
            Rule::string => {
                arg.description = normalize_parsed(arg_pair.as_str());
            }
            Rule::data_type => match arg_pair.as_str() {
                "bool" => {
                    arg.data_type = DataType::Boolean;
                }
                "number" => {
                    arg.data_type = DataType::Number;
                }
                "string" => {
                    arg.data_type = DataType::String;
                }
                _ => {
                    arg.data_type = DataType::Unknown;
                }
            },
            _ => {
                // println!("missed case: {}", arg_pair);
            }
        }
    }
    arg
}

fn parse_group(pair: Pair<Rule>) -> ArgumentGroup {
    let mut arg_group = ArgumentGroup {
        id: "".to_string(),
        items: vec![],
    };
    let mut items = Vec::new();
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::ident => {
                arg_group.id = inner_pair.as_str().to_owned();
            }
            Rule::word_list => {
                for identifier in inner_pair.into_inner() {
                    items.push(identifier.as_str().to_owned());
                }
            }
            _ => {
                // ignore
            }
        }
    }
    arg_group.items = items;
    arg_group
}

fn parse_project_arguments(pair: Pair<Rule>) -> Vec<ProjectArgument> {
    let mut args = Vec::new();
    for inner_pair in pair.into_inner() {
        let arg = parse_argument(inner_pair);
        args.push(arg);
    }
    args
}

fn get_file_content(file_name: String) -> Result<String, String> {
    match fs::read_to_string(file_name) {
        Ok(content) => Ok(content),
        Err(e) => Err(format!("{}", e)),
    }
}

/// status_code rule parser
fn parse_status_code(pair: Pair<Rule>) -> StatusCode {
    let mut status_code = StatusCode {
        code: "0".to_string(),
        description: "".to_string(),
        is_retryable: false,
    };
    for sub_rule in pair.into_inner() {
        match sub_rule.as_rule() {
            Rule::string => {
                status_code.description = normalize_parsed(sub_rule.as_str());
            }
            Rule::status_code_n => {
                status_code.code = sub_rule.as_str().to_owned();
            }
            Rule::status_codes_options => {
                if let " retryable" = sub_rule.as_str() {
                    status_code.is_retryable = true;
                }
            }
            _ => {
                // println!("ignoring sub rule {:?}", sub_rule);
            }
        }
    }
    status_code
}

fn parse_api_operation(pair: Pair<Rule>, project: &Project) -> (HttpMethod, APIConfiguration) {
    let mut definition = APIConfiguration {
        description: String::new(),
        operation: String::new(),
        query_string: vec![],
        path_params: vec![],
        headers: vec![],
        tags: vec![],
        status_codes: vec![],
        produces: vec![],
        consumes: vec![],
        use_cases: vec![],
        example: String::new(),
    };
    let mut current_method = HttpMethod::Unknown;
    for api_pair in pair.into_inner() {
        match api_pair.as_rule() {
            Rule::http_verb => {
                match api_pair.as_str().to_ascii_lowercase().as_str() {
                    "get" => {
                        current_method = HttpMethod::Get;
                    }
                    "post" => {
                        current_method = HttpMethod::Post;
                    }
                    "put" => {
                        current_method = HttpMethod::Put;
                    }
                    "delete" => {
                        current_method = HttpMethod::Delete;
                    }
                    "patch" => {
                        current_method = HttpMethod::Patch;
                    }
                    _ => {
                        // ignore
                    }
                }
            }
            Rule::api_params => {
                let mut current_keyword = "";
                for param in api_pair.into_inner() {
                    match param.as_rule() {
                        Rule::api_use_cases => {
                            for use_case in param.into_inner() {
                                definition
                                    .use_cases
                                    .push(normalize_parsed(use_case.as_str()));
                            }
                        }
                        Rule::api_single_option => {
                            for single_opt in param.into_inner() {
                                match single_opt.as_rule() {
                                    Rule::api_single_keyword => {
                                        current_keyword = single_opt.as_str();
                                    }
                                    Rule::word_list => {
                                        let mut definitions = Vec::new();
                                        let inner_text = single_opt.as_str();
                                        for param_value in single_opt.into_inner() {
                                            match param_value.as_rule() {
                                                Rule::ident => {
                                                    definitions.push(normalize_parsed(
                                                        param_value.as_str(),
                                                    ));
                                                }
                                                Rule::group_reference => {
                                                    // parse
                                                    // needs an array of string, of all elements contained in group
                                                    let name = param_value.into_inner().as_str();
                                                    let values =
                                                        project.spread_group(current_keyword, name);
                                                    for v in values {
                                                        definitions
                                                            .push(normalize_parsed(v.as_str()));
                                                    }
                                                }
                                                _ => {
                                                    // ignore
                                                }
                                            }
                                        }
                                        match current_keyword {
                                            "params" => {
                                                for def in definitions {
                                                    definition.path_params.push(def);
                                                }
                                            }
                                            "query" => {
                                                for def in definitions {
                                                    definition.query_string.push(def);
                                                }
                                            }
                                            "headers" => {
                                                for def in definitions {
                                                    definition.headers.push(def);
                                                }
                                            }
                                            "tags" => {
                                                for def in definitions {
                                                    definition.tags.push(def);
                                                }
                                            }
                                            "produces" => {
                                                for def in definitions {
                                                    definition.produces.push(def);
                                                }
                                            }
                                            "consumes" => {
                                                for def in definitions {
                                                    definition.consumes.push(def);
                                                }
                                            }
                                            "example" => {
                                                definition.example = normalize_parsed(inner_text);
                                            }
                                            _ => {
                                                // not place to attach
                                            }
                                        }
                                    }
                                    _ => {
                                        // println!("ignoring unknown option {:?}", single_opt);
                                    }
                                }
                            }
                        }
                        Rule::api_operation => {
                            for op in param.into_inner() {
                                let normalized = normalize_parsed(op.as_str());
                                if !normalized.is_empty() {
                                    definition.operation = normalized;
                                }
                            }
                        }
                        Rule::api_status_codes => {
                            // parse group
                            for inner in param.into_inner() {
                                match inner.as_rule() {
                                    Rule::word_list => {
                                        for status_component in inner.into_inner() {
                                            match status_component.as_rule() {
                                                Rule::ident => {
                                                    let code = normalize_parsed(status_component.as_str());
                                                    if !code.is_empty() {
                                                        definition.status_codes.push(code);
                                                    }
                                                },
                                                Rule::group_reference => {
                                                    let name = status_component.into_inner().as_str();
                                                    let values =
                                                        project.spread_group("status_codes", name);
                                                    for v in values {
                                                        definition.status_codes
                                                            .push(normalize_parsed(v.as_str()));
                                                    }
                                                },
                                                _ => {
                                                //
                                                }
                                            }
                                        }
                                    },
                                    _ => {

                                    }
                                }
                            }

                        }
                        _ => {
                            // println!("ignoring param {:?}", param);
                        }
                    }
                }
            }
            Rule::string => {
                definition.description = normalize_parsed(api_pair.as_str());
            }
            _ => {
                // println!("ignoring {:?}", api_pair);
            }
        }
    }
    (current_method, definition)
}

/// API rule parser
fn parse_api(pair: Pair<Rule>, project: &Project) -> APIWrapper {
    let mut wrapper = APIWrapper {
        endpoint: "".to_string(),
        definition: APIDefinition {
            get: None,
            post: None,
            put: None,
            delete: None,
            patch: None,
        },
    };
    let mut current_endpoint = String::new();
    for api_sub_rule in pair.into_inner() {
        match api_sub_rule.as_rule() {
            Rule::path => {
                for sub_path in api_sub_rule.into_inner() {
                    current_endpoint += sub_path.as_str();
                }
                wrapper.endpoint = current_endpoint.to_owned();
            }
            Rule::api_op => {
                let definition = parse_api_operation(api_sub_rule, project);
                match definition.0 {
                    HttpMethod::Get => {
                        wrapper.definition.get = Some(definition.1);
                    }
                    HttpMethod::Post => {
                        wrapper.definition.post = Some(definition.1);
                    }
                    HttpMethod::Put => {
                        wrapper.definition.put = Some(definition.1);
                    }
                    HttpMethod::Delete => {
                        wrapper.definition.delete = Some(definition.1);
                    }
                    HttpMethod::Patch => {
                        wrapper.definition.patch = Some(definition.1);
                    }
                    _ => {}
                }
            }
            _ => {
                // println!("skipping {:?}", api_sub_rule);
            }
        }
    }
    wrapper
}

/// Trims spaces and double quote characters from strings
fn normalize_parsed(source: &str) -> String {
    let mut normalized = source.trim().to_owned();
    let d_quote = "\"";
    if normalized.starts_with(d_quote) && normalized.ends_with(d_quote) {
        normalized.remove(normalized.len() - 1);
        normalized.remove(0);
    }
    normalized
}

/// Parses the top level language keywords
fn parse_value(pair: Pair<Rule>, project: &mut Project) {
    match pair.as_rule() {
        Rule::api_file => {
            pair.into_inner().for_each(|local_pair| {
                parse_value(local_pair, project);
            });
        }
        Rule::spec_header => {
            for p in pair.into_inner() {
                match p.as_rule() {
                    Rule::spec_header_title => {
                        let title = normalize_parsed(p.into_inner().as_str());
                        project.title = title;
                    }
                    Rule::spec_header_version => {
                        let version = normalize_parsed(p.into_inner().as_str());
                        project.version = version;
                    }
                    _ => {
                        // println!("ignoring sub rule {:?}", p);
                    }
                }
            }
        }
        Rule::spec_items => {
            let mut n = String::new();
            for p in pair.into_inner() {
                if n.is_empty() {
                    n = p.as_str().to_owned();
                }
                let args = parse_project_arguments(p);
                match n.as_str() {
                    "headers:" => {
                        project.headers = args;
                    }
                    "params:" => {
                        project.params = args;
                    }
                    "query:" => {
                        project.query = args;
                    }
                    _ => {
                        // ignore the rest
                    }
                }
            }
        }
        Rule::status_codes => {
            for sc in pair.into_inner() {
                if let Rule::status_code_desc = sc.as_rule() {
                    project.status_codes.push(parse_status_code(sc));
                }
            }
        }
        Rule::apis => {
            for api in pair.into_inner() {
                let wrapped_api = parse_api(api, project);
                project
                    .endpoints
                    .insert(wrapped_api.endpoint, wrapped_api.definition);
            }
        }
        Rule::common_groups_def => {
            let placeholder = "unk";
            let mut target = placeholder;
            let mut args = Vec::new();
            for it in pair.into_inner() {
                if target == placeholder {
                    target = it.as_str();
                    continue;
                }
                args.push(parse_group(it));
            }
            match target {
                "headers_groups" => {
                    project.headers_groups = args;
                }
                "params_groups" => {
                    project.params_groups = args;
                }
                "query_groups" => {
                    project.query_groups = args;
                }
                "status_codes_groups" => {
                    project.status_codes_groups = args;
                }
                _ => {
                    // ignoring target
                }
            }
        }
        _ => {
            // println!("Top level rule ignored: {:?}", pair);
        }
    }
}

#[test]
fn test_item_parser() {
    let valid_expressions = vec![
        " x-my-auth string alias auth required (default_value): \"It does something\"\n",
        " x-my-optional string alias opt1: \"If present, something will happen\"\n",
        " id string: \"Unique identifier\"\n",
        " filter bool: \"Possible values name date\"\n",
    ];
    let expected_args = vec![
        ProjectArgument::new("x-my-auth", DataType::String, "auth", true, "default_value"),
        ProjectArgument::new("x-my-optional", DataType::String, "opt1", false, ""),
        ProjectArgument::new("id", DataType::String, "", false, ""),
        ProjectArgument::new("filter", DataType::Boolean, "", false, ""),
    ];
    let mut current_case = 0;
    for expr in valid_expressions {
        let mut pair = ApishParser::parse(Rule::item, expr).unwrap();
        let arg = parse_argument(pair.next().unwrap());
        let expected = expected_args.get(current_case).unwrap();
        current_case += 1;
        assert_eq!(&arg, expected);
    }
}
