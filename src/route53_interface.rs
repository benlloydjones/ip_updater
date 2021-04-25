use rusoto_core::{Region, request};
use rusoto_credential::EnvironmentProvider;
use rusoto_route53::{
    Change,
    ChangeBatch,
    ChangeResourceRecordSetsRequest,
    ListResourceRecordSetsRequest,
    ResourceRecord,
    ResourceRecordSet,
    Route53,
    Route53Client,
};
use std::env;
use std::error::Error;
use std::net::Ipv4Addr;

#[path = "ip_finder.rs"] mod ip_finder;

enum RecordUpdateKind { CREATE, DELETE }

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
    record_sets: Vec<ResourceRecordSet>
) -> Result<Vec<Ipv4Addr>, Box<dyn Error>> {
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

pub fn get_route_53_client() -> Route53Client {
    let request_dispatcher = request::HttpClient::new().unwrap();
    let credentials_provider = EnvironmentProvider::with_prefix("IP_UPDATER");
    let region = Region::UsEast1;
    Route53Client::new_with(
        request_dispatcher, credentials_provider, region,
    )
}

#[tokio::main]
pub async fn get_current_a_record(client: &Route53Client) -> Result<Vec<Ipv4Addr>, Box<dyn Error>> {
    let list_resource_record_sets_response = Route53Client::list_resource_record_sets(
        client, get_list_resource_record_sets_request()?,
    ).await?;

    Ok(
        get_current_a_record_addresses(
            list_resource_record_sets_response.resource_record_sets
        )?
    )
}

fn get_update_a_record_request(ip_address: Ipv4Addr, action: RecordUpdateKind) -> Result<ChangeResourceRecordSetsRequest, Box<dyn Error>> {
    let name = env::var("IP_UPDATER_DOMAIN_NAME")?;
    let hosted_zone_id = env::var("IP_UPDATER_HOSTED_ZONE_ID")?;
    let change_batch = ChangeBatch {
        changes: vec![Change {
            action: match action {
                RecordUpdateKind::CREATE => "CREATE".to_string(),
                RecordUpdateKind::DELETE => "DELETE".to_string(),
            },
            resource_record_set: ResourceRecordSet {
                alias_target: None,
                failover: None,
                geo_location: None,
                health_check_id: None,
                multi_value_answer: None,
                name: name,
                region: None,
                resource_records: Some(vec![ResourceRecord { value: ip_address.to_string() }]),
                set_identifier: None,
                ttl: Some(60),
                traffic_policy_instance_id: None,
                type_: "A".to_string(),
                weight: None,
            }
        }],
        comment: None,
    };
    Ok(
        ChangeResourceRecordSetsRequest {
            change_batch: change_batch,
            hosted_zone_id: hosted_zone_id,
        }
    )
}

#[tokio::main]
pub async fn update_a_records_on_route53(
    client: &Route53Client,
    new_ip_address: Ipv4Addr,
    old_ip_address: Ipv4Addr,
) -> Result<(), Box<dyn Error>> {
    let delete_a_record_request = get_update_a_record_request(old_ip_address, RecordUpdateKind::DELETE)?;
    Route53::change_resource_record_sets(client, delete_a_record_request).await?;
    let create_a_record_request = get_update_a_record_request(new_ip_address, RecordUpdateKind::CREATE)?;
    Route53Client::change_resource_record_sets(client, create_a_record_request).await?;
    Ok(())
}
