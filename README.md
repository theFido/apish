## Goals

- API DSL that allows reusing items
- Extended metadata: i.e. use cases
- WASM plugins to render

Rules:
- Every list above `apis` is reusable items

```
headers:
	'x-my-auth' string alias auth required (its default value): "It does something"
	'x-my-optional' string alias opt1: "If present, something will happen"
params:
	id string: Unique identifier
query:
	filter string: Possible values name, date
apis:
	/api/contact:
		get:
			headers: auth opt1
			use cases:
				- Get all contacts
		post:
			headers: auth
			use cases:
				- Create a new contact
			produces: json				
	/api/contact/{id}:
		get:
			headers: auth
			params: id
```