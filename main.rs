#![feature(core_intrinsics)]
#![feature(pattern)]
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, prelude::*};
use std::str::FromStr;
use std::time::Instant;
use std::path::Path;
use std::env;
use std::process::exit;


#[derive(Debug)]
struct Event {
    latitude: f32,
    longitude: f32,
    date: String,
}

impl FromStr for Event {
    type Err = Box<dyn Error>;
    
    fn from_str(s: &str) -> Result<Self, Box<dyn Error>> {
        let fields: Vec<&str> = s.split(" ").collect();
        if fields.len() < 11 {
            println!("{:?}", fields);
            return Err("Invalid event data".into());
        }
        
        let latitude = fields[8].parse()?;
        let longitude = fields[9].parse()?;
        let date = fields[1..4].join(",").to_string();
        
        Ok(Event {
            latitude,
            longitude,
            date,
        })
    }
}

fn point_within_square(center: (f32, f32), event: &Event, area_squared: f32) -> bool {
    let distance_from_center_km = f32::sqrt(area_squared)/2.0;
    let d_degrees = distance_from_center_km / 111.0;
    
    let min_latitude = center.0 - d_degrees;
    let min_longitude = center.1 - d_degrees;
    let max_latitude = center.0 + d_degrees;
    let max_longitude = center.1 + d_degrees;
    
    event.latitude >= min_latitude && event.latitude <= max_latitude && event.longitude >= min_longitude && event.longitude <= max_longitude
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if !args.len() == 3 {
        println!("Expected 3 arguments, {} found.", args.len());
        exit(-1);
    }
    let center_latitude: f32 = 59.9323673548189;
    let center_longitude: f32 = 10.984623367099006;
    let area_squared: f32 = 10.0;
    let (year_start, year_end) = (2001, 2023);
    
    let now: Instant = Instant::now();
    println!("Latitude: {} Longitude: {} Kvadratkilometer dekket: {}", center_latitude, center_longitude, area_squared);
    
    let mut files: Vec<String> = Vec::new();
    
    
    for year in year_start..year_end {
        files.push(format!("data/{}.txt", year))
    }
    let path: String = format!("Latitude: {}, Longitude: {}, Areal: {}km2.txt", center_latitude, center_longitude, area_squared);
    let mut data_file = OpenOptions::new()
        .create_new(!Path::new(&path).exists())
        .append(true)
        .open(&path)?;
    
    if let Err(e) = writeln!(data_file, "År : Antall nedslag : Nedslag/areal : Antall dager") {
        eprintln!("Couldn't write to file: {}", e)
    }
    
    for file in files {
        let year: String = String::from(&file[5..9].to_string());
        println!("{}", year);
        let current_file = File::open(file)?;
        let reader = BufReader::new(current_file);
    
        let mut dates: Vec<String> = Vec::new();
        let mut strikes: f32 = 0.0;
        
        for line in reader.lines() {
            let event = line?.parse::<Event>()?;
            
            if point_within_square(
                (center_latitude, center_longitude),
                &event,
                area_squared
            ) {
                if !dates.iter().any(|d| *d == event.date) {
                    dates.push(event.date.clone());
                }
                strikes += 1.0;
            }
        }
        if strikes > 0.0 {
            let line = format!("{} {} {} {}\n", year, strikes, strikes/area_squared, dates.len());
            if let Err(e) = data_file.write_all(line.as_bytes()) {
                eprintln!("Couldn't write to file: {}", e);
            }
        }
        
    }
    
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
    Ok(())
}
