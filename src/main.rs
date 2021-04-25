extern crate reqwest;

use reqwest::StatusCode;

fn main() {
    match get_ip_address() {
        Ok(text) => println!("the ip address is: {}", text),
        Err(text) => println!("{}", text),
    }
}

fn get_ip_address() -> Result<String, String> {
    match reqwest::blocking::get("https://api.ipify.org") {
        Ok(response) => match response.status() {
            StatusCode::OK => match response.text() {
                Ok(text) => Ok(text),
                Err(_) => Err(String::from("The response was not text")),
            },
            _ => Err(String::from("Status Code was not OK.")),
        }
        Err(_) => Err(String::from("Request failed.")),
    }
}

