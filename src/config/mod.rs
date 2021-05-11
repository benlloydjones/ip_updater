use std::env;
use std::error::Error;
use std::net::Ipv4Addr;

pub struct Config {
    pub domains: Vec<Domain>,
}

pub struct Domain {
    pub hosted_zone_id: String,
    pub domain_name: String,
    pub ip_addresses: Vec<Ipv4Addr>,
}

pub fn get_config() -> Result<Config, Box<dyn Error>> {
    let zone_names: Vec<Domain> = env::var("IP_UPDATER_ZONE_NAMES")?.split(",").map(|zone_name| {
        let zone_name: Vec<&str> = zone_name.split("/").collect();
        Domain {
            hosted_zone_id: zone_name[0].to_string(),
            domain_name: zone_name[1].to_string(),
            ip_addresses: vec![],
        }
    }).collect();

    Ok(Config{ domains: zone_names })
}
