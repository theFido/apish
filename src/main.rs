enum DataType {
	String(String),
	Number(String),
	Boolean(String),
}

struct Argument {
	name: String,
	description: String,
	data_type: DataType,
	required: bool,
	default_value: String,
}

struct StatusCode {
	code: isize,
	description: String,
}

struct API {
	parent_path: String,
	sub_path: String,
	use_cases: Vec<String>,
	params: Vec<Argument>,
	query_strings: Vec<Argument>,
	headers: Vec<Argument>,
	status_codes: Vec<StatusCode>,
}

impl API {
	fn new() -> API {
		API {
			parent_path: "a".to_string(),
			sub_path: "".to_string(),
			use_cases: vec![],
			params: vec![],
			query_strings: vec![],
			headers: vec![],
			status_codes: vec![]
		}
	}
}

fn main() {
	let api = API::new();
  println!("Hello, world! {}", api.parent_path);
}
