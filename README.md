# APIsh

OpenAPI pre-processor heavily inspired by [Stylus](https://stylus-lang.com) and 
[Sass](https://sass-lang.com).

The `.api` file produces written with this DSL produces two artifacts:

- `api.json` Is a JSON representation of the DSL, ideal to automate tasks like
  code generation where you can also define common elements.
- `api-spec.json` Is a JSON representation of each API composed with the common 
  elements, ideal to produce final documentation formats like OpenAPI.

🙊 Proudly written in [Rust](https://www.rust-lang.org) 🦀.

## Motivation

Writing OpenAPI files is not fun when you have a bunch of repetitive items, 
APIsh introduces variables for all the common objects (status codes, headers, 
etc.) and reduces the need for repetitive name fields.

## Goals

- API DSL that allows reusing items
- Extended metadata: i.e. use cases
- WASM plugins to render

Rules:
- Every list above `apis` is reusable items

Example (example.api):
```
headers:
  x-my-auth string alias auth required: "It does something"
  x-auth-two string alias auth2: "Other auth option"
  x-my-optional string alias opt1 (i_am_optional): "If present, something will happen"
params:
  id string: "Unique identifier"
query:
  filter string: "Possible values name date"
  offset number: "Page offset"
status_codes:
  401: "Token not provided"
  424 retryable: "Temporary failure"
headers_groups:
  allAuth: auth auth2
  other: auth
query_groups:
  paging: filter offset

// starting api
apis:
  /api/contact:
    get: "Returns all contact information"
      headers: auth opt1
      produces: json
      query: g(paging)
      operation: wolf
      status_codes: 401 424 200 201
      use_cases:
        "Does something"
        "Does something else"
    post: "Creates a new contact entry"
      headers: g(allAuth) opt1
      produces: json
      consumes: json
      status_codes: 200
      example: example_api_1
  /api/contact/{id}:
    get: "Something else"
        headers: opt1
        params: id
        produces: json
        status_codes: 200
```

ToDo:
- [ ] Add imports
- [X] Groups
- [ ] Produce postman
- [ ] Links section
- [ ] JSON 5 - JSON with examples
- [ ] Plugins
- [ ] Swagger
- [ ] Grammar documentation 
- [X] Status codes
- [X] File watcher to monitor changes in source `api` file
- [X] Comments
- [X] Use cases
- [X] Produces/consumes
- [X] Example

NFR ToDo:
- [ ] Debug