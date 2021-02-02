# APIsh

OpenAPI pre-processor heavily inspired by [Stylus](https://stylus-lang.com) and 
[Sass](https://sass-lang.com).

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
  x-my-optional string alias opt1 (i_am_optional): "If present, something will happen"
params:
  id string: "Unique identifier"
query:
  filter string: "Possible values name date"
status_codes:
  401: "Token not provided"
  424 retryable: "Temporary failure"

// starting api
apis:
  /api/contact:
    get: "Returns all contact information"
      operation: getContact
      headers: auth opt1
      produces: json
      query: filter
      status_codes: 401 424 200
      use_cases:
        "Does something"
        "Does something else"
    post: "Creates a new contact entry"
      headers: auth
      produces: json
      consumes: json
      status_codes: 200
  /api/contact/{id}:
    get: "Something else"
        headers: opt1
        params: id
        produces: json
        status_codes: 200

```

ToDo:
- [ ] Add imports
- [X] Status codes
- [ ] Groups
- [X] Comments
- [X] Use cases
- [X] Produces/consumes

NFR ToDo:
- [ ] Debug