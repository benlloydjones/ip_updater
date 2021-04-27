extern crate reqwest;
extern crate rusoto_core;
extern crate rusoto_credential;
extern crate rusoto_route53;

mod ip_finder;
mod route53_interface;

use std::process;

fn main() {
    let current_external_ip_address = match ip_finder::get_ip_address() {
       Ok(ip_address) => ip_address,
       Err(error_message) => {
           eprintln!("{}", error_message);
           process::exit(1);
       }
    };

    let client = route53_interface::get_route_53_client();

    let current_a_records = match route53_interface::get_current_a_record(&client) {
        Ok(addresses) => addresses,
        Err(error_message) => {
            eprintln!("{}", error_message);
            process::exit(1);
        },
    };

    if vec![current_external_ip_address] == current_a_records {
        println!(
            "A records up to date with current external IP address: {}.\n\nExiting process",
            current_external_ip_address.to_string()
        );
        process::exit(0);
    }

    match route53_interface::update_a_records_on_route53(
        &client,
        current_external_ip_address,
        current_a_records[0],
    ) {
        Ok(_) => println!("Update complete.\n\nExiting process"),
        Err(error_message) => {
            eprintln!("{}", error_message);
            process::exit(1);
        }
    };
    process::exit(0);
}
