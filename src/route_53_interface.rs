use rusoto_core::{Region, request};
use rusoto_credential::EnvironmentProvider;
use rusoto_route53::{
    ListResourceRecordSetsResponse,
    ListResourceRecordSetsRequest,
    Route53,
    Route53Client,
};
use std::env;
use std::error::Error;
use std::net::Ipv4Addr;

#[path = "ip_finder.rs"] mod ip_finder;

fn get_list_resource_record_sets_request() -> Result<
    ListResourceRecordSetsRequest, Box<dyn Error>
> {
    let hosted_zone_id = env::var("IP_UPDATER_HOSTED_ZONE_ID")?;
    Ok(ListResourceRecordSetsRequest {
        hosted_zone_id: hosted_zone_id,
        max_items: None,
        start_record_identifier: None,
        start_record_name: None,
        start_record_type: None,
    })
}

fn get_current_a_record_addresses(
    record_sets_response: ListResourceRecordSetsResponse
) -> Result<Vec<Ipv4Addr>, Box<dyn Error>> {
    let record_sets = record_sets_response.resource_record_sets;
    let mut current_a_records = Vec::new();
    
    for record_set in record_sets.iter() {
        if record_set.type_ != "A" { continue }
        match &record_set.resource_records {
            Some(resource_records) => for resource_record in resource_records.iter() {
                let record_as_ipv4_enum = ip_finder::string_to_ipv4(
                    resource_record.value.clone()
                )?;
                current_a_records.push(record_as_ipv4_enum);
            },
            None => continue,
        }
    }
    
    Ok(current_a_records)
}

#[tokio::main]
pub async fn get_current_a_record() -> Result<Vec<Ipv4Addr>, Box<dyn Error>> {
    let dispatcher = request::HttpClient::new()?;
    let client = Route53Client::new_with(
        dispatcher,
        EnvironmentProvider::with_prefix("IP_UPDATER"),
        Region::UsEast1,
    );
    let list_resource_record_sets_request = get_list_resource_record_sets_request()?;
    let list_resource_record_sets_response = Route53Client::list_resource_record_sets(
        &client, list_resource_record_sets_request,
    ).await?;
    let current_ip_addresses = get_current_a_record_addresses(list_resource_record_sets_response)?;
    Ok(current_ip_addresses)
}