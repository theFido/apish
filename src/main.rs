mod examples;
mod open_api;
mod project;
mod models;

extern crate structopt;
#[macro_use]
extern crate structopt_derive;

extern crate serde;
extern crate serde_json;

use serde_json::Value;

extern crate pest;
#[macro_use]
extern crate pest_derive;

use std::collections::HashMap;
use std::fs::File;
use std::ops::Deref;
use std::sync::mpsc::channel;
use std::time::Duration;

use crate::project::ProjectArgument;
use serde::Serialize;
use structopt::StructOpt;

use notify::{RecommendedWatcher, RecursiveMode, Watcher};

use notify::DebouncedEvent::Write;
use project::{APIConfiguration, APIDefinition, DataType, Project, StatusCode};
use crate::models::{Entity, Enum, ProjectModel};

#[derive(StructOpt)]
struct Opt {
    #[structopt(short = "w", help = "Keeps watching source API file changes")]
    watch: bool,
    #[structopt(short = "f", help = "Input file")]
    input: String,
    #[structopt(short = "m", help = "Input models file", default_value = "./models.model")]
    models_file: String,
    #[structopt(short = "o", help = "Output file", default_value = "./api.json")]
    output: String,
    #[structopt(
    short = "a",
    help = "Open API spec file",
    default_value = "./openapi.json"
    )]
    open_api: String,
    #[structopt(
    short = "e",
    help = "Examples json file",
    default_value = "./example.json"
    )]
    examples: String,
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
struct APIEndpoint {
    description: String,
    operation: String,
    use_cases: Vec<String>,
    params: Vec<Argument>,
    query_strings: Vec<Argument>,
    headers: Vec<Argument>,
    tags: Vec<String>,
    status_codes: Vec<StatusCode>,
    produces: Vec<String>,
    consumes: Vec<String>,
    example: Option<Vec<examples::Example>>,
    request_object: Option<Entity>,
    request_enum: Option<Enum>,
    response_object: Option<Entity>,
    response_enum: Option<Enum>,
}

#[derive(Debug, Serialize)]
struct APISpec {
    title: String,
    version: String,
    spec: HashMap<String, API>,
    models: Option<ProjectModel>,
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

impl APIEndpoint {
    fn argument_from_project(argument: &ProjectArgument) -> Argument {
        Argument {
            name: argument.name.clone(),
            description: argument.description.clone(),
            required: argument.required,
            data_type: argument.data_type.clone(),
            default_value: argument.default_value.clone(),
        }
    }

    fn transform_arguments(arguments: Vec<&ProjectArgument>) -> Vec<Argument> {
        let mut args = Vec::new();
        for argument in arguments {
            let arg = APIEndpoint::argument_from_project(argument);
            args.push(arg);
        }
        args
    }

    fn get_arguments(section: &str, project: &Project, from_list: &[String]) -> Vec<Argument> {
        let args = Vec::new();
        match section {
            "headers" => {
                let headers = project.get_headers(from_list);
                return APIEndpoint::transform_arguments(headers);
            }
            "query" => {
                let query_strings = project.get_query_strings(from_list);
                return APIEndpoint::transform_arguments(query_strings);
            }
            "params" => {
                let params = project.get_path_params(from_list);
                return APIEndpoint::transform_arguments(params);
            }
            _ => {
                // invalid section name
            }
        }
        args
    }

    fn new_from_api_configuration(
        configuration: &Option<APIConfiguration>,
        project: &Project,
    ) -> Option<APIEndpoint> {
        match configuration {
            Some(config) => {
                let mut request: Option<Entity> = None;
                let mut request_enum: Option<Enum> = None;
                let mut response: Option<Entity> = None;
                let mut response_enum: Option<Enum> = None;
                let mut ex: Option<Vec<examples::Example>> = None;
                if let Some(example) = project.examples.get(config.example.as_str()) {
                    ex = Some(example.clone());
                }
                match &project.models {
                    Some(models) => {
                        if let Some(m) = models.entities.get(config.request_model.as_str()) {
                            request = Some(m.clone());
                        }
                        if let Some(m) = models.enums.get(config.request_model.as_str()) {
                            request_enum = Some(m.clone());
                        }
                        if let Some(m) = models.entities.get(config.response_model.as_str()) {
                            response = Some(m.clone());
                        }
                        if let Some(m) = models.enums.get(config.response_model.as_str()) {
                            response_enum = Some(m.clone());
                        }
                    }
                    _ => {
                        // ignore
                    }
                }
                let endpoint = APIEndpoint {
                    description: config.description.to_owned(),
                    operation: config.operation.to_owned(),
                    use_cases: config.use_cases.clone(),
                    params: APIEndpoint::get_arguments("params", &project, &config.path_params),
                    query_strings: APIEndpoint::get_arguments(
                        "query",
                        &project,
                        &config.query_string,
                    ),
                    headers: APIEndpoint::get_arguments("headers", &project, &config.headers),
                    tags: config.tags.clone(),
                    status_codes: project.get_status_codes(&config.status_codes),
                    produces: get_mime_types(&config.produces),
                    consumes: get_mime_types(&config.consumes),
                    example: ex,
                    request_object: request,
                    request_enum,
                    response_object: response,
                    response_enum,
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

    fn new_project_spec(project: &Project) -> APISpec {
        let mut api = HashMap::new();
        for endpoint in &project.endpoints {
            let api_path = endpoint.0.to_owned();
            let api_def = API::new_from_api_definition(&endpoint.1, &project);
            api.insert(api_path, api_def);
        }
        let mut models: Option<ProjectModel> = None;
        if let Some(x) = &project.models {
            models = Some(x.clone());
        }
        APISpec {
            title: project.title.to_owned(),
            version: project.version.to_owned(),
            spec: api,
            models,
        }
    }
}

pub fn get_mime_types(list: &[String]) -> Vec<String> {
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
        ("mp4".to_owned(), "video/mp4".to_owned()),
        // image formats
        ("jpg".to_owned(), "image/jpeg".to_owned()),
        ("png".to_owned(), "image/png".to_owned()),
        ("svg".to_owned(), "image/svg+xml".to_owned()),
    ]
        .iter()
        .cloned()
        .collect();
    for item in list {
        let trimmed = item.trim().to_owned();
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

fn produce_files(
    source: &str,
    models_source: &str,
    examples_source: &str,
    output: &str,
    spec_output: &str,
    open_api_output: &str,
) {
    let failure_icon = "🧟";

    match Project::new_from_file(source.to_string(), models_source.to_string(), examples_source.to_string()) {
        Ok(project) => {
            // producing api.json
            let file = File::create(output).unwrap();
            serde_json::to_writer(file, &project).unwrap();

            // producing api-spec.json (from project)
            let api = API::new_project_spec(&project);
            let api_file = File::create(spec_output).unwrap();
            serde_json::to_writer(api_file, &api).unwrap();

            // producing openapi.json
            let open_api = open_api::OpenAPI::new_from_project_spec(&project);
            let open_api_file = File::create(open_api_output).unwrap();
            serde_json::to_writer(open_api_file, &open_api).unwrap();
            println!(
                "✅ Generated {}, {}, and {}",
                output, spec_output, open_api_output
            );
        }
        Err(e) => {
            println!("{} {}", failure_icon, e);
        }
    }
}

fn watch(
    source: &str,
    examples_source: &str,
    models_source: &str,
    output: &str,
    spec_output: &str,
    open_api: &str,
) -> notify::Result<()> {
    let (tx, rx) = channel();
    let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(1))?;
    watcher.watch(source, RecursiveMode::NonRecursive)?;
    loop {
        match rx.recv() {
            Ok(event) => {
                if let Write(_) = event {
                    produce_files(source, models_source, examples_source, output, spec_output, open_api);
                }
            }
            Err(e) => println!("{:?}", e),
        }
    }
}

fn main() {
    let opt = Opt::from_args();
    let version = env!("CARGO_PKG_VERSION");
    println!("APIsh 🙊 v{}\nReading API from {}", version, opt.input);
    if opt.watch {
        println!("Listening for changes in {} file", opt.input);
        if let Err(e) = watch(
            opt.input.as_ref(),
            opt.models_file.as_ref(),
            opt.examples.as_ref(),
            opt.output.as_ref(),
            opt.spec_output.as_ref(),
            opt.open_api.as_ref(),
        ) {
            println!("Error listening: {:?}", e);
        }
    } else {
        produce_files(
            opt.input.as_ref(),
            opt.models_file.as_ref(),
            opt.examples.as_ref(),
            opt.output.as_ref(),
            opt.spec_output.as_ref(),
            opt.open_api.as_ref(),
        );
    }
}
