use std::error;

use chrono::{Duration, Local, NaiveDateTime};
use rusqlite::{params, Connection};
use weather_fetcher::idokep_client::IdokepClient;
use weather_fetcher::WeatherClient;

#[derive(Debug)]
struct WeatherSample {
    id: i32,
    source: String,
    timestamp: NaiveDateTime,
    temperature: f64,
    precipitation: f64,
    precipitation_24h: f64,
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let mut idokep_client = IdokepClient::new();
    let data = idokep_client.fetch_weather_data();
    println!("{:?}", data);
    let conn = Connection::open("weather.db")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS weather_samples (
            id INTEGER PRIMARY KEY,
            source TEXT NOT NULL,
            timestamp DATETIME NOT NULL,
            temperature REAL NOT NULL,
            precipitation REAL NOT NULL,
            precipitation_24h REAL NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "INSERT INTO weather_samples (source, timestamp, temperature, precipitation, precipitation_24h) values (?1, ?2, ?3, ?4, ?5)",
        params![String::from("idokep"), Local::now().naive_local(), data.temperature, data.precipitation, data.precipitation_24h])?;

    let mut stmt = conn.prepare("SELECT * FROM weather_samples WHERE DATETIME(timestamp) > ?")?;
    let samples = stmt.query_map(
        params![Local::now().naive_local() - Duration::minutes(3)],
        |row| {
            Ok(WeatherSample {
                id: row.get(0)?,
                source: row.get(1)?,
                timestamp: row.get(2)?,
                temperature: row.get(3)?,
                precipitation: row.get(4)?,
                precipitation_24h: row.get(5)?,
            })
        },
    )?;

    for sample in samples {
        println!("Found sample {:?}", sample.unwrap());
    }

    Ok(())
}
