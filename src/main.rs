#![allow(clippy::upper_case_acronyms)]

use blake2::{Blake2b512, Digest};
use chrono::*;
use serde::Deserialize;
use std::fs::{self, File};
use std::io::{self, prelude::*};
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
struct Config {
    input_dir: String,
    output_dir: String,
    time_limit: u8,
    only_copy: bool,
    station: Station,
}

#[derive(Debug, Deserialize)]
struct Station {
    line: u8,
    name: SType,
}

#[derive(Debug, Deserialize)]
enum SType {
    SPI,
    AOI,
}

impl SType {
    fn as_str(&self) -> String {
        match self {
            SType::SPI => "SPI".to_string(),
            SType::AOI => "AOI".to_string(),
        }
    }
}

// Loading the config file.
const CONFIG_FILE: &str = ".\\config.toml";
fn load_config() -> Config {
    let mut file = File::open(CONFIG_FILE).expect("Unable to open config file!");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Failed to read config file!");

    println!("{}", contents);

    let config: Config =
        toml::from_str(contents.as_str()).expect("Failed to parse TOML config file!");

    println!("{:?}", config);

    config
}

// Checking if the folders exist
fn sanity_check(config: &Config) -> bool {
    assert!(
        Path::new(&config.input_dir).exists(),
        "Input path \"{}\" does NOT exist!",
        config.input_dir
    );
    assert!(
        Path::new(&config.output_dir).exists(),
        "Output path \"{}\" does NOT exist!",
        config.output_dir
    );

    true
}

// Goes through every subfolder, and checks any of the files is older then the set limit. If they are, then it calls move_file() with them.
fn check_folder(config: &Config, directory: PathBuf) -> Result<(), std::io::Error> {
    let time_now = Local::now();
    let time_limit = Duration::try_days(config.time_limit as i64).unwrap();

    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            check_folder(config, path)?;
        } else {
            match path.metadata() {
                Err(err) => panic!("Could not get file metadata! {err}"),
                Ok(meta) => {
                    let modification_time: DateTime<Local> = meta
                        .modified()
                        .expect("Could not read modification date!")
                        .into();
                    let time_difference = time_now - modification_time;

                    if time_difference > time_limit {
                        println!("{:?} - {:?}", path, modification_time);
                        move_file(config, path, modification_time)?;
                    }
                }
            }
        }
    }

    Ok(())
}

// File moving with checksum checking.
fn move_file(config: &Config, file: PathBuf, time: DateTime<Local>) -> Result<(), std::io::Error> {
    // Generating output folder.
    // [Station_name/year/month/day/] where Station_name is [L+line_number+_+AOI/SPI]
    let station_str = format!("L{}_{}", config.station.line, config.station.name.as_str());
    let output_folder = format!(
        "{}\\{}\\{}\\{}\\{}\\",
        config.output_dir,
        station_str,
        time.year(),
        time.month(),
        time.day()
    );
    println!("\t{output_folder}");

    // 1 - Check if destination directory exists, if not, then create it.
    let mut output_path = PathBuf::from(&output_folder);
    if !output_path.exists() {
        fs::create_dir_all(&output_path)?;
    }

    // Append filename to the destination directory
    output_path.push(file.file_name().expect("Failed to extract file name!"));
    println!("\t{:?}", output_path);

    // 2 - Calculate checksum
    let mut hasher = Blake2b512::new();
    let _n = io::copy(&mut fs::File::open(&file)?, &mut hasher)?;
    let hash = hasher.finalize();
    //println!("\t{:?}", hash);

    // 3 - Copy file
    fs::copy(&file, &output_path)?;

    // 4 - Verify checksum
    hasher = Blake2b512::new();
    let _n = io::copy(&mut fs::File::open(&output_path)?, &mut hasher)?;
    let hash_2 = hasher.finalize();

    if hash == hash_2 {
        // 5 - Remove original file
        if config.only_copy {
            println!("\t\tChecksum is OK!");
        } else {
            println!("\t\tChecksum is OK! Removing the original file!");
            fs::remove_file(file)?;
        }
    } else {
        // Remove duplicate if checksum failed
        println!("\t\tERROR: checksum is NOK! Removing copied file!");
        fs::remove_file(output_path)?;
    }

    Ok(())
}

fn main() -> Result<(), std::io::Error> {
    let config = load_config();

    if sanity_check(&config) {
        let path = PathBuf::from(config.input_dir.clone());
        check_folder(&config, path)?;
    }

    Ok(())
}
