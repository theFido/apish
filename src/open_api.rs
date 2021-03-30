use crate::get_mime_types;
use crate::project::{APIConfiguration, APIDefinition, Project, ProjectArgument, StatusCode};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize)]
pub struct OpenAPI {
    openapi: String,
    info: InfoSpec,
    paths: HashMap<String, PathSpec>,
}

#[derive(Debug, Serialize)]
pub struct PathSpec {
    #[serde(skip_serializing_if = "Option::is_none")]
    get: Option<APISpec>,
    #[serde(skip_serializing_if = "Option::is_none")]
    post: Option<APISpec>,
    #[serde(skip_serializing_if = "Option::is_none")]
    put: Option<APISpec>,
    #[serde(skip_serializing_if = "Option::is_none")]
    delete: Option<APISpec>,
    #[serde(skip_serializing_if = "Option::is_none")]
    patch: Option<APISpec>,
}

#[derive(Debug, Serialize)]
pub struct APISpec {
    pub description: String,
    #[serde(rename = "operationId")]
    pub operation_id: String,
    pub parameters: Vec<APIParamSpec>,
    pub responses: HashMap<String, APIResponseSpec>,
}

#[derive(Debug, Serialize)]
pub struct APIResponseSpec {
    pub description: String,
    pub content: HashMap<String, APIResponseContentSpec>,
}

#[derive(Debug, Serialize, Clone)]
pub struct APIResponseContentSpec {
    pub schema: APISchemaSpec,
}

#[derive(Debug, Serialize, Clone)]
pub struct APISchemaSpec {
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Debug, Serialize)]
pub struct APIParamSpec {
    pub name: String,
    #[serde(rename = "in")]
    pub where_in: String,
    pub description: String,
    #[serde(skip_serializing_if = "is_not_false")]
    pub required: bool,
    pub schema: APIParamSchemaSpec,
}

#[derive(Debug, Serialize)]
pub struct APIParamSchemaSpec {
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Debug, Serialize)]
pub struct InfoSpec {
    title: String,
    version: String,
}

fn is_not_false(a_value: &bool) -> bool {
    !*a_value
}

fn args_to_params(list: Vec<&ProjectArgument>, group_name: &str) -> Vec<APIParamSpec> {
    let mut params: Vec<APIParamSpec> = Vec::new();
    for item in list {
        let param = APIParamSpec {
            name: item.name.to_string(),
            where_in: group_name.to_string(),
            description: item.description.to_string(),
            required: item.required,
            schema: APIParamSchemaSpec {
                type_field: item.data_type.as_str().to_owned(),
            },
        };
        params.push(param);
    }
    params
}

fn empty_response() -> APIResponseContentSpec {
    APIResponseContentSpec {
        schema: APISchemaSpec {
            type_field: "string".to_string(),
        },
    }
}

fn status_codes_to_response_spec(
    status_codes: Vec<StatusCode>,
    produces: &[String],
) -> HashMap<String, APIResponseSpec> {
    let mut codes = HashMap::new();
    let mut content_produces = HashMap::new();
    for p in produces {
        content_produces.insert(p.to_owned(), empty_response());
    }
    for sc in status_codes {
        let content = APIResponseSpec {
            description: sc.description,
            content: content_produces.clone(),
        };
        codes.insert(sc.code.to_string(), content);
    }
    codes
}

fn get_spec_from_endpoint(
    endpoint: &Option<APIConfiguration>,
    project: &Project,
) -> Option<APISpec> {
    match endpoint {
        Some(definition) => {
            let mut params: Vec<APIParamSpec> = Vec::new();

            let mut headers = args_to_params(project.get_headers(&definition.headers), "header");
            params.append(&mut headers);
            let mut query =
                args_to_params(project.get_query_strings(&definition.query_string), "query");
            params.append(&mut query);
            let mut path_params =
                args_to_params(project.get_path_params(&definition.path_params), "path");
            params.append(&mut path_params);

            let responses = status_codes_to_response_spec(
                project.get_status_codes(&definition.status_codes),
                &get_mime_types(&definition.produces),
            );
            let spec = APISpec {
                description: definition.description.to_owned(),
                operation_id: definition.operation.to_owned(),
                parameters: params,
                responses,
            };
            Some(spec)
        }
        None => None,
    }
}

fn api_spec_from_endpoint(endpoint: &APIDefinition, project: &Project) -> PathSpec {
    PathSpec {
        get: get_spec_from_endpoint(&endpoint.get, project),
        post: get_spec_from_endpoint(&endpoint.post, project),
        put: get_spec_from_endpoint(&endpoint.put, project),
        delete: get_spec_from_endpoint(&endpoint.delete, project),
        patch: get_spec_from_endpoint(&endpoint.patch, project),
    }
}

fn get_paths_from_project(project: &Project) -> HashMap<String, PathSpec> {
    let mut response = HashMap::new();
    for (endpoint, definition) in &project.endpoints {
        response.insert(
            endpoint.to_owned(),
            api_spec_from_endpoint(definition, &project),
        );
    }
    response
}

impl OpenAPI {
    pub fn new_from_project_spec(project: &Project) -> OpenAPI {
        OpenAPI {
            openapi: "3.0.3".to_string(),
            info: InfoSpec {
                title: project.title.to_owned(),
                version: project.version.to_owned(),
            },
            paths: get_paths_from_project(&project),
        }
    }
}
