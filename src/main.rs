extern crate reqwest;

use reqwest::StatusCode;
use std::net::Ipv4Addr;

fn main() {
    match get_ip_address() {
        Ok(ip_address) => println!("{}", ip_address),
        Err(text) => eprintln!("{}", text),
    }
}

fn get_ip_address<'a>() -> Result<Ipv4Addr, &'a str> {
    match reqwest::blocking::get("https://api.ipify.org") {
        Ok(response) => match response.status() {
            StatusCode::OK => match response.text() {
                Ok(text) => string_to_ipv4(text),
                Err(_) => Err("The response was not text"),
            },
            _ => Err("Status Code was not OK."),
        },
        Err(_) => Err("Request failed."),
    }
}

fn string_to_ipv4<'a>(ip_as_string: String) -> Result<Ipv4Addr, &'a str> {
    let string_digits: Vec<&str> = ip_as_string.split(".").collect();
    if string_digits.len() < 4 {
        return Err("Not enough parts of IP address.");
    }

    let mut digits: Vec<u8> = Vec::new();
    for string_digit in string_digits.iter() {
        match string_digit.parse::<u8>() {
            Ok(digit) => digits.push(digit),
            Err(_) => return Err("Unable to parse IP address."),
        }
    }

    Ok(Ipv4Addr::new(digits[0], digits[1], digits[2], digits[3]))
}
