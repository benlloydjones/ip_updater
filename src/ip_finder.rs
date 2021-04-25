use reqwest;
use std::error::Error;
use std::fmt;
use std::net::Ipv4Addr;

#[derive(Debug)]
struct VecNotLongEnough(String);

impl fmt::Display for VecNotLongEnough {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "There is an error: {}", self.0)
    }
}

impl Error for VecNotLongEnough {}

#[tokio::main]
pub async fn get_ip_address () -> Result<Ipv4Addr, Box<dyn Error>> {
    let response = reqwest::get("https://api.ipify.org").await?.text().await?;
    let result = string_to_ipv4(response)?;
    Ok(result)
}

pub fn string_to_ipv4 (ip_as_string: String) -> Result<Ipv4Addr, Box<dyn Error>> {
    let string_digits: Vec<&str> = ip_as_string.split(".").collect();
    let mut digits: Vec<u8> = Vec::new();
    for string_digit in string_digits.iter() {
        let parsed_digit = string_digit.parse::<u8>()?;
        digits.push(parsed_digit);
    }
    if digits.len() < 4 {
        return Err(Box::new(VecNotLongEnough("Not enough digits for IP address".into())));
    }
    let result = Ipv4Addr::new(digits[0], digits[1], digits[2], digits[3]);
    Ok(result)
}
