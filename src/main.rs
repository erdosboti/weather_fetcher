use std::error;

use weather_fetcher::idokep_client::IdokepClient;
use weather_fetcher::WeatherClient;

fn main() -> Result<(), Box<dyn error::Error>> {
    let mut idokep_client = IdokepClient::new();
    println!("{:?}", idokep_client.fetch_weather_data());
    Ok(())
}
