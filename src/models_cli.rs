use std::fs;
use std::fs::File;

mod models;


extern crate pest;
#[macro_use]
extern crate pest_derive;

extern crate structopt;
#[macro_use]
extern crate structopt_derive;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Opt {
    #[structopt(short = "f", help = "Input file")]
    input: String,
    #[structopt(short = "o", help = "Output file", default_value = "./models.json")]
    output: String,
}

fn main() {
    let opt = Opt::from_args();
    let version = env!("CARGO_PKG_VERSION");
    println!("Models 🚀 v{}\nReading models from {}", version, opt.input);
    if let Ok(content) = fs::read_to_string(opt.input) {
        let model_file = models::get_models(&content);
        let api_file = File::create(&opt.output).unwrap();
        serde_json::to_writer(api_file, &model_file).unwrap();
        println!("Generated {} models file", opt.output);
    } else {
        println!("Invalid input");
    }
}