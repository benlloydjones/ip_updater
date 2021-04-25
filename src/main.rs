extern crate reqwest;
extern crate rusoto_core;
extern crate rusoto_route53;

mod ip_finder;
mod route_53_interface;

use std::process;

fn main() {
    let current_external_ip_address = match ip_finder::get_ip_address() {
       Ok(ip_address) => ip_address,
       Err(error_message) => {
           eprintln!("{}", error_message);
           process::exit(1);
       }
    };

    let current_a_records = match route_53_interface::get_current_a_record() {
        Ok(addresses) => addresses,
        Err(error_message) => {
            eprintln!("{}", error_message);
            process::exit(1);
        },
    };
    println!("Current external ip address: {:?}", current_external_ip_address);
    for address in current_a_records.iter() {
        println!("A record address: {:?}", address);
    }
}
