extern crate structopt;
#[macro_use]
extern crate structopt_derive;

extern crate serde;
extern crate serde_json;

extern crate pest;
#[macro_use]
extern crate pest_derive;

use std::collections::HashMap;
use std::fs;
use std::fs::File;

use pest::iterators::Pair;
use pest::Parser;

use structopt::StructOpt;

use serde::Serialize;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct ApishParser;

#[derive(Debug, Serialize)]
enum DataType {
    String,
    Number,
    Boolean,
    Unknown,
}

enum PairModifiers {
    Alias,
    Unknown,
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

#[derive(StructOpt)]
struct Opt {
    #[structopt(short = "f", help = "Input file")]
    input: String,
    #[structopt(short = "o", help = "Output file", default_value = "./api.json")]
    output: String,
    #[structopt(
        short = "s",
        help = "Spec output file (OpenAPI superset)",
        default_value = "./api-spec.json"
    )]
    spec_output: String,
}

#[derive(Debug, Serialize)]
struct Argument {
    name: String,
    description: String,
    data_type: DataType,
    required: bool,
    default_value: String,
}

#[derive(Debug, Serialize)]
struct StatusCode {
    code: String,
    description: String,
    is_retryable: bool,
}

#[derive(Debug, Serialize)]
struct APIEndpoint {
    description: String,
    operation: String,
    use_cases: Vec<String>,
    params: Vec<Argument>,
    query_strings: Vec<Argument>,
    headers: Vec<Argument>,
    status_codes: Vec<StatusCode>,
    produces: Vec<String>,
    consumes: Vec<String>,
}

#[derive(Debug, Serialize)]
struct API {
    #[serde(skip_serializing_if = "Option::is_none")]
    get: Option<APIEndpoint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    post: Option<APIEndpoint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    put: Option<APIEndpoint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    delete: Option<APIEndpoint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    patch: Option<APIEndpoint>,
}

#[derive(Debug, Serialize)]
struct ProjectArgument {
    name: String,
    data_type: DataType,
    alias: String,
    required: bool,
    default_value: String,
    description: String,
}

#[derive(Debug, Serialize)]
struct Project {
    headers: Vec<ProjectArgument>,
    query: Vec<ProjectArgument>,
    params: Vec<ProjectArgument>,
    status_codes: Vec<StatusCode>,
    endpoints: HashMap<String, APIDefinition>,
}

#[derive(Debug, Serialize)]
struct APIConfiguration {
    description: String,
    operation: String,
    query_string: Vec<String>,
    path_params: Vec<String>,
    headers: Vec<String>,
    status_codes: Vec<String>,
    produces: Vec<String>,
    consumes: Vec<String>,
    use_cases: Vec<String>,
}

#[derive(Debug, Serialize)]
struct APIDefinition {
    #[serde(skip_serializing_if = "Option::is_none")]
    get: Option<APIConfiguration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    post: Option<APIConfiguration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    put: Option<APIConfiguration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    delete: Option<APIConfiguration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    patch: Option<APIConfiguration>,
}

// ToDo: Replace me
#[derive(Debug)]
struct APIWrapper {
    endpoint: String,
    definition: APIDefinition,
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

impl Project {
    fn new() -> Project {
        Project {
            headers: vec![],
            query: vec![],
            params: vec![],
            status_codes: vec![],
            endpoints: HashMap::new(),
        }
    }

    fn new_from_file(file_name: String) -> Result<Project, String> {
        let api_definition = get_file_content(file_name);
        match ApishParser::parse(Rule::api_file, &api_definition.as_ref()) {
            Ok(mut pairs) => {
                let n = pairs.next();
                match n {
                    Some(pair) => {
                        let mut project = Project::new();
                        parse_value(pair, &mut project);
                        return Ok(project);
                    }
                    None => {
                        println!("Did not find any API element");
                        return Err("Cannot process file".to_string());
                    }
                }
            }
            Err(e) => return Err(format!("Cannot parse API definition {}", e)),
        }
    }

    fn get_header(&self, name: &String) -> Option<Argument> {
        for header in &self.headers {
            let owned_name = name.trim().clone();
            if header.alias == owned_name || header.name == owned_name {
                let arg = Argument {
                    name: header.name.clone(),
                    description: header.description.clone(),
                    data_type: header.data_type.clone(),
                    required: header.required,
                    default_value: header.default_value.clone(),
                };
                return Some(arg);
            }
        }
        None
    }

    fn get_query_string(&self, name: &String) -> Option<Argument> {
        for query in &self.query {
            let owned_name = name.trim().clone();
            if query.alias == owned_name || query.name == owned_name {
                let arg = Argument {
                    name: query.name.clone(),
                    description: query.description.clone(),
                    data_type: query.data_type.clone(),
                    required: query.required,
                    default_value: query.default_value.clone(),
                };
                return Some(arg);
            }
        }
        None
    }

    fn get_path_param(&self, name: &String) -> Option<Argument> {
        for path_param in &self.params {
            let owned_name = name.trim().clone();
            if path_param.alias == owned_name || path_param.name == owned_name {
                let arg = Argument {
                    name: path_param.name.clone(),
                    description: path_param.description.clone(),
                    data_type: path_param.data_type.clone(),
                    required: path_param.required,
                    default_value: path_param.default_value.clone(),
                };
                return Some(arg);
            }
        }
        None
    }

    fn get_status_code(&self, status_code: &String) -> Option<StatusCode> {
        for code in &self.status_codes {
            let owned_name = status_code.clone();
            if code.code == owned_name {
                let project_status = StatusCode {
                    code: code.code.to_string(),
                    description: code.description.to_string(),
                    is_retryable: code.is_retryable,
                };
                return Some(project_status);
            }
        }
        match status_code.as_str() {
            "200" => Some(StatusCode::new(status_code.as_str(), "Ok")),
            "201" => Some(StatusCode::new(status_code.as_str(), "Created")),
            "202" => Some(StatusCode::new(status_code.as_str(), "Accepted")),
            "204" => Some(StatusCode::new(status_code.as_str(), "No Content")),
            "400" => Some(StatusCode::new(status_code.as_str(), "Bad Request")),
            "401" => Some(StatusCode::new(status_code.as_str(), "Unauthorized")),
            "403" => Some(StatusCode::new(status_code.as_str(), "Forbidden")),
            "404" => Some(StatusCode::new(status_code.as_str(), "Not Found")),
            "405" => Some(StatusCode::new(status_code.as_str(), "Method Not Allowed")),
            "408" => Some(StatusCode::new(status_code.as_str(), "Request Timeout")),
            "413" => Some(StatusCode::new(status_code.as_str(), "Payload Too Large")),
            "415" => Some(StatusCode::new(
                status_code.as_str(),
                "Unsupported Media Type",
            )),
            "424" => Some(StatusCode::new(status_code.as_str(), "Failed Dependency")),
            "429" => Some(StatusCode::new(status_code.as_str(), "Too Many Requests")),
            _ => Some(StatusCode::new(status_code.as_str(), "")),
        }
    }

    fn get_headers(&self, list: &Vec<String>) -> Vec<Argument> {
        let mut headers = Vec::new();
        for item in list {
            match self.get_header(item) {
                Some(header) => {
                    headers.push(header);
                }
                None => {}
            }
        }
        headers
    }

    fn get_query_strings(&self, list: &Vec<String>) -> Vec<Argument> {
        let mut query_strings = Vec::new();
        for item in list {
            match self.get_query_string(item) {
                Some(qs) => {
                    query_strings.push(qs);
                }
                None => {}
            }
        }
        query_strings
    }

    fn get_path_params(&self, list: &Vec<String>) -> Vec<Argument> {
        let mut path_params = Vec::new();
        for item in list {
            match self.get_path_param(item) {
                Some(path_param) => {
                    path_params.push(path_param);
                }
                None => {}
            }
        }
        path_params
    }

    fn get_status_codes(&self, list: &Vec<String>) -> Vec<StatusCode> {
        let mut codes = Vec::new();
        for status in list {
            match self.get_status_code(status) {
                Some(sc) => {
                    codes.push(sc);
                }
                None => {}
            }
        }
        codes
    }
}

impl APIEndpoint {
    fn new_from_api_configuration(
        configuration: &Option<APIConfiguration>,
        project: &Project,
    ) -> Option<APIEndpoint> {
        match configuration {
            Some(config) => {
                let endpoint = APIEndpoint {
                    description: config.description.to_owned(),
                    operation: config.operation.to_owned(),
                    use_cases: config.use_cases.clone(),
                    params: project.get_path_params(&config.path_params),
                    query_strings: project.get_query_strings(&config.query_string),
                    headers: project.get_headers(&config.headers),
                    status_codes: project.get_status_codes(&config.status_codes),
                    produces: get_mime_types(&config.produces),
                    consumes: get_mime_types(&config.consumes),
                };
                Some(endpoint)
            }
            None => None,
        }
    }
}

impl API {
    fn new_from_api_definition(def: &APIDefinition, project: &Project) -> API {
        API {
            get: APIEndpoint::new_from_api_configuration(&def.get, project),
            post: APIEndpoint::new_from_api_configuration(&def.post, project),
            put: APIEndpoint::new_from_api_configuration(&def.put, project),
            delete: APIEndpoint::new_from_api_configuration(&def.delete, project),
            patch: APIEndpoint::new_from_api_configuration(&def.patch, project),
        }
    }

    fn new_from_project(project: Project) -> HashMap<String, API> {
        let mut api = HashMap::new();
        for endpoint in &project.endpoints {
            let api_path = endpoint.0.to_owned();
            let api_def = API::new_from_api_definition(&endpoint.1, &project);
            api.insert(api_path, api_def);
        }
        api
    }
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
    fn as_str(&self) -> &'static str {
        match *self {
            DataType::String => "string",
            DataType::Number => "number",
            DataType::Boolean => "bool",
            DataType::Unknown => "unk",
        }
    }
}

impl PartialEq for DataType {
    fn eq(&self, other: &Self) -> bool {
        if self.as_str() == other.as_str() {
            return true;
        }
        return false;
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

fn get_mime_types(list: &Vec<String>) -> Vec<String> {
    let mut mime_types = Vec::new();
    let mime_map: HashMap<String, String> = [
        ("json".to_owned(), "application/json".to_owned()),
        ("xml".to_owned(), "application/xml".to_owned()),
        ("text".to_owned(), "text/plain".to_owned()),
        ("css".to_owned(), "text/css".to_owned()),
        ("html".to_owned(), "text/html".to_owned()),
        ("javascript".to_owned(), "application/javascript".to_owned()),
        ("js".to_owned(), "application/javascript".to_owned()),
        ("multipart".to_owned(), "multipart/form-data".to_owned()),
        ("binary".to_owned(), "application/octet-stream".to_owned()),
    ]
    .iter()
    .cloned()
    .collect();
    for item in list {
        let trimmed = item.trim().clone().to_owned();
        match mime_map.get(&trimmed) {
            Some(mime_type) => {
                mime_types.push(mime_type.to_owned());
            }
            None => {
                mime_types.push(trimmed.clone());
            }
        }
    }
    mime_types
}

fn get_file_content(file_name: String) -> String {
    match fs::read_to_string(file_name) {
        Ok(content) => content,
        _ => {
            return "".to_string();
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
                                    Rule::pair_modifiers => match k.as_str() {
                                        "alias" => {
                                            key = PairModifiers::Alias;
                                        }
                                        _ => {
                                            println!("unk pair modifier {}", k.as_str());
                                        }
                                    },
                                    Rule::ident => {
                                        value = k.as_str().to_owned();
                                    }
                                    _ => {
                                        println!("skipped mod_pair {}", k);
                                    }
                                }
                            }
                            match key {
                                PairModifiers::Alias => {
                                    arg.alias = value;
                                }
                                _ => {
                                    // ignore
                                }
                            }
                        }
                        Rule::single_modifiers => match opt.as_str() {
                            "required" => {
                                arg.required = true;
                            }
                            _ => {
                                println!("Single modifier unknown: '{}'", opt.as_str());
                            }
                        },
                        Rule::default_value => {
                            for dv_inner in opt.into_inner() {
                                match dv_inner.as_rule() {
                                    Rule::ident => {
                                        arg.default_value = dv_inner.as_str().to_owned();
                                    }
                                    _ => {
                                        // do nothing
                                    }
                                }
                            }
                        }
                        _ => {
                            println!("skipped option {}", opt);
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
                println!("missed case: {}", arg_pair);
            }
        }
    }
    arg
}

fn parse_project_arguments(pair: Pair<Rule>) -> Vec<ProjectArgument> {
    let mut args = Vec::new();
    for inner_pair in pair.into_inner() {
        let arg = parse_argument(inner_pair);
        args.push(arg);
    }
    args
}

fn parse_status_code(pair: Pair<Rule>) -> Result<StatusCode, String> {
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
                match sub_rule.as_str() {
                    " retryable" => {
                        status_code.is_retryable = true;
                    }
                    _ => {
                        // ignoring
                    }
                }
            }
            _ => {
                // println!("ignoring sub rule {:?}", sub_rule);
            }
        }
    }
    Ok(status_code)
}

fn parse_api_operation(pair: Pair<Rule>) -> Option<(HttpMethod, APIConfiguration)> {
    let mut definition = APIConfiguration {
        description: "".to_string(),
        operation: "".to_string(),
        query_string: vec![],
        path_params: vec![],
        headers: vec![],
        status_codes: vec![],
        produces: vec![],
        consumes: vec![],
        use_cases: vec![],
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
                                        match current_keyword {
                                            "params" => {
                                                definition
                                                    .path_params
                                                    .push(normalize_parsed(single_opt.as_str()));
                                            }
                                            "query" => {
                                                definition
                                                    .query_string
                                                    .push(normalize_parsed(single_opt.as_str()));
                                            }
                                            "headers" => {
                                                definition
                                                    .headers
                                                    .push(normalize_parsed(single_opt.as_str()));
                                            }
                                            "produces" => {
                                                definition
                                                    .produces
                                                    .push(normalize_parsed(single_opt.as_str()));
                                            }
                                            "consumes" => {
                                                definition
                                                    .consumes
                                                    .push(normalize_parsed(single_opt.as_str()));
                                            }
                                            _ => {
                                                // not place to attach
                                            }
                                        }
                                    }
                                    _ => {
                                        println!("ignoring unknown option {:?}", single_opt);
                                    }
                                }
                            }
                        }
                        Rule::api_operation => {
                            for op in param.into_inner() {
                                definition.operation = normalize_parsed(op.as_str());
                            }
                        }
                        Rule::api_status_codes => {
                            for status in param.into_inner() {
                                let code = normalize_parsed(status.as_str());
                                definition.status_codes.push(code);
                            }
                        }
                        _ => {
                            println!("ignoring param {:?}", param);
                        }
                    }
                }
            }
            Rule::string => {
                definition.description = normalize_parsed(api_pair.as_str());
            }
            _ => {
                println!("ignoring {:?}", api_pair);
            }
        }
    }
    Some((current_method, definition))
}

fn parse_api(pair: Pair<Rule>) -> APIWrapper {
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
                match parse_api_operation(api_sub_rule) {
                    Some(definition) => match definition.0 {
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
                    },
                    None => {
                        // ignore
                    }
                }
            }
            _ => {
                println!("skipping {:?}", api_sub_rule);
            }
        }
    }
    wrapper
}

fn normalize_parsed(source: &str) -> String {
    let mut normalized = source.trim().clone().to_owned();
    let d_quote = "\"";
    if normalized.starts_with(d_quote) && normalized.ends_with(d_quote) {
        normalized.remove(normalized.len() - 1);
        normalized.remove(0);
    }
    normalized
}

fn parse_value(pair: Pair<Rule>, project: &mut Project) {
    match pair.as_rule() {
        Rule::api_file => {
            pair.into_inner().for_each(|local_pair| {
                parse_value(local_pair, project);
            });
        }
        Rule::spec_items => {
            let mut n = String::new();
            for p in pair.into_inner() {
                if n == "" {
                    n = p.as_str().to_owned();
                }
                let rule_name = p.as_str();
                match rule_name {
                    "headers:" => {
                        // ignore the rule name
                    }
                    "params:" => {
                        // ignore the rule name
                    }
                    "query:" => {
                        // ignore the rule name
                    }
                    _ => {
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
                                println!("skipping rule option {}", n);
                            }
                        }
                    }
                }
            }
        }
        Rule::item_list => {
            // println!("[top] item list! - {}", pair);
        }
        Rule::keyword => {
            // println!("[top] keyword");
        }
        Rule::status_codes => {
            for sc in pair.into_inner() {
                match sc.as_rule() {
                    Rule::status_code_desc => {
                        match parse_status_code(sc) {
                            Ok(status_code) => {
                                project.status_codes.push(status_code);
                            }
                            Err(_) => {
                                // ignoring invalid status code
                            }
                        }
                    }
                    _ => {
                        // println!("ignoring status code rule: {:?}", sc);
                    }
                }
            }
        }
        Rule::apis => {
            for api in pair.into_inner() {
                let wrapped_api = parse_api(api);
                project
                    .endpoints
                    .insert(wrapped_api.endpoint, wrapped_api.definition);
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

fn main() {
    let failure_icon = "🧟";
    let opt = Opt::from_args();
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");
    println!("APIsh 🙊 v{}", VERSION);
    println!("Reading API from {}", opt.input);
    match Project::new_from_file(opt.input) {
        Ok(project) => {
            let file = File::create(&opt.output).unwrap();
            serde_json::to_writer(file, &project).unwrap();
            let api = API::new_from_project(project);
            let api_file = File::create(&opt.spec_output).unwrap();
            serde_json::to_writer(api_file, &api).unwrap();
            println!("Generated {} and {}", opt.output, opt.spec_output);
        }
        Err(e) => {
            println!("{} {}", failure_icon, e);
        }
    }
}
