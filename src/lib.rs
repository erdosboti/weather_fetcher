pub mod idokep_client;

#[derive(Debug)]
pub struct WeatherData {
    pub temperature: f64,
    pub precipitation: f64,
    pub precipitation_24h: f64,
}

pub trait WeatherClient {
    fn fetch_weather_data(&mut self) -> WeatherData;
}
