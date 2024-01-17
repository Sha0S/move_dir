use serde::Deserialize;
use std::fs::File;
use std::io::prelude::*;


#[derive(Debug, Deserialize)]
struct Config {
    input_dir: String,
    output_dir: String,
    time_limit: u8,
    station: Station
}

#[derive(Debug, Deserialize)]
struct Station {
    line: u8,
    name: SType
}

#[derive(Debug, Deserialize)]
enum SType { SPI, AOI}

const config_file: &str = ".\\config.toml";
fn load_config() -> Config {
    let mut file = File::open(config_file).expect("Unable to open config file!");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Failed to read config file!");

    println!("{}", contents);


    let config: Config = toml::from_str(contents.as_str()).expect("");

    println!("{:?}", config);

    config
}


fn main() {
    load_config();
}
