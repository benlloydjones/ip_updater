extern crate chrono;
extern crate openssl_probe;
extern crate reqwest;
extern crate rusoto_core;
extern crate rusoto_credential;
extern crate rusoto_route53;

mod config;
mod ip_finder;
pub mod route53_interface;

use std::process;

use chrono;

fn main() {
    openssl_probe::init_ssl_cert_env_vars();
    let now = chrono::Utc::now();

    let mut config: config::Config = match config::get_config() {
        Ok(config) => config,
        Err(_) => {
            eprint!("{} : Badly formatted IP_UPDATER_ZONE_NAMES\n", now);
            process::exit(1);
        }
    };

    let current_external_ip_address = match ip_finder::get_ip_address() {
        Ok(ip_address) => ip_address,
        Err(error_message) => {
            eprint!("{} : {}\n", now, error_message);
            process::exit(1);
        }
    };

    let client = route53_interface::get_route_53_client();

    config = route53_interface::get_a_records_for_domains(&client, config);

    for domain in config.domains.into_iter() {
        if vec![current_external_ip_address] == domain.ip_addresses {
            print!(
                "{} : A records for {} up to date with current external IP address: {}.\n",
                now,
                domain.domain_name,
                current_external_ip_address.to_string()
            );
            continue;
        }
        
        match route53_interface::update_a_records_on_route53(
            &client,
            current_external_ip_address,
            domain.ip_addresses[0],
            &domain.hosted_zone_id,
            &domain.domain_name,
        ) {
            Ok(_) => {
                print!(
                    "{} : A records for {} updated to current external IP address {}.\n",
                    now,
                    domain.domain_name,
                    current_external_ip_address.to_string(),
                )
            },
            Err(error_message) => {
                eprint!("{}: {}\n", now, error_message);
                process::exit(1);
            }
        };
    }
    process::exit(0);
}
