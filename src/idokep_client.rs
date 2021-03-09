mod bit_utils;
mod number_recognition;
mod png;

use base64::decode;
use regex::Regex;
use std::error;

use crate::WeatherClient;
use crate::WeatherData;
use number_recognition::NumberRecognition;
use png::Png;

static AUTOMATA_URL: &'static str = "https://www.idokep.hu/automata/globallhotel";
static TEMPERATURE_IMAGE_REGEXP: &'static str = r#"<th class="">Hőmérséklet</th>\s+<td><img alt="Embedded Image" src="data:image/png;base64,(.+)"> °C</td>"#;
// TODO: change this to the correct url
static PRECIPITATION_IMAGE_REGEXP: &'static str = r#"<th>Harmatpont</th>\s+<td><img alt="Embedded Image" src="data:image/png;base64,(.+)"> °C</td>"#;

pub struct IdokepClient<'a> {
    automata_url: &'a str,
    page_body: Option<String>,
}

impl<'a> IdokepClient<'a> {
    pub fn new() -> Self {
        Self {
            automata_url: AUTOMATA_URL,
            page_body: None,
        }
    }

    fn fetch_page(&mut self) -> Result<(), Box<dyn error::Error>> {
        self.page_body = Some(ureq::get(self.automata_url).call()?.into_string()?);
        Ok(())
    }

    fn extract_data(&mut self, data_type: &str) -> Result<f64, Box<dyn error::Error>> {
        if self.page_body.is_none() {
            self.fetch_page()?;
        }
        let decoded_image = match data_type {
            "temperature" => self.get_decoded_image(TEMPERATURE_IMAGE_REGEXP),
            "precipitation" => self.get_decoded_image(PRECIPITATION_IMAGE_REGEXP),
            "precipitation_24h" => Ok(Vec::new()),
            _ => Ok(Vec::new()),
        };
        if let Ok(decoded_image) = decoded_image {
            NumberRecognition::extract_number(Png::from_bytes(decoded_image))
        } else {
            Err("image couldn't be extracted")?
        }
    }

    fn get_decoded_image(&self, regexp: &str) -> Result<Vec<u8>, Box<dyn error::Error>> {
        let re = Regex::new(regexp);
        let encoded_image = re?
            .captures(self.page_body.as_ref().unwrap())
            .ok_or("no captures")?
            .get(1)
            .ok_or("no results")?
            .as_str();
        Ok(decode(encoded_image)?)
    }
}

impl<'a> WeatherClient for IdokepClient<'a> {
    fn fetch_weather_data(&mut self) -> WeatherData {
        WeatherData {
            temperature: self.extract_data("temperature").unwrap_or(0.0),
            precipitation: self.extract_data("precipitation").unwrap_or(0.0),
            precipitation_24h: 0.0, // self.extract_precipitation_24h(),
        }
    }
}
