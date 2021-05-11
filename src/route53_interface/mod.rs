use rusoto_core::{request, Region};
use rusoto_credential::EnvironmentProvider;
use rusoto_route53::{
    Change, ChangeBatch, ChangeResourceRecordSetsRequest, ListResourceRecordSetsRequest,
    ResourceRecord, ResourceRecordSet, Route53, Route53Client,
};
use std::error::Error;
use std::net::Ipv4Addr;
use std::process;

enum RecordUpdateKind {
    CREATE,
    DELETE,
}

fn get_list_resource_record_sets_request(hosted_zone_id: &String) -> Result<ListResourceRecordSetsRequest, Box<dyn Error>>
{
    Ok(ListResourceRecordSetsRequest {
        hosted_zone_id: hosted_zone_id.clone(),
        max_items: None,
        start_record_identifier: None,
        start_record_name: None,
        start_record_type: None,
    })
}

fn get_current_a_record_addresses(
    record_sets: Vec<ResourceRecordSet>,
) -> Result<Vec<Ipv4Addr>, Box<dyn Error>> {
    let mut current_a_records = Vec::new();
    for record_set in record_sets.iter() {
        if record_set.type_ != "A" {
            continue;
        }
        match &record_set.resource_records {
            Some(resource_records) => {
                for resource_record in resource_records.iter() {
                    let record_as_ipv4_enum =
                        super::ip_finder::string_to_ipv4(resource_record.value.clone())?;
                    current_a_records.push(record_as_ipv4_enum);
                }
            }
            None => continue,
        }
    }
    Ok(current_a_records)
}

pub fn get_route_53_client() -> Route53Client {
    let request_dispatcher = request::HttpClient::new().unwrap();
    let credentials_provider = EnvironmentProvider::with_prefix("IP_UPDATER");
    let region = Region::UsEast1;
    Route53Client::new_with(request_dispatcher, credentials_provider, region)
}

#[tokio::main]
async fn get_current_a_record(client: &Route53Client, hosted_zone_id: &String) -> Result<Vec<Ipv4Addr>, Box<dyn Error>> {
    let list_resource_record_sets_response =
        Route53Client::list_resource_record_sets(client, get_list_resource_record_sets_request(hosted_zone_id)?)
            .await?;

    Ok(get_current_a_record_addresses(
        list_resource_record_sets_response.resource_record_sets,
    )?)
}

fn get_update_a_record_request(
    ip_address: Ipv4Addr,
    action: RecordUpdateKind,
    hosted_zone_id: &String,
    name: &String,
) -> Result<ChangeResourceRecordSetsRequest, Box<dyn Error>> {
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
                name: name.clone(),
                region: None,
                resource_records: Some(vec![ResourceRecord {
                    value: ip_address.to_string(),
                }]),
                set_identifier: None,
                ttl: Some(60),
                traffic_policy_instance_id: None,
                type_: "A".to_string(),
                weight: None,
            },
        }],
        comment: None,
    };
    Ok(ChangeResourceRecordSetsRequest {
        change_batch: change_batch,
        hosted_zone_id: hosted_zone_id.clone(),
    })
}

#[tokio::main]
pub async fn update_a_records_on_route53(
    client: &Route53Client,
    new_ip_address: Ipv4Addr,
    old_ip_address: Ipv4Addr,
    hosted_zone_id: &String,
    name: &String,
) -> Result<(), Box<dyn Error>> {
    let delete_a_record_request =
        get_update_a_record_request(old_ip_address, RecordUpdateKind::DELETE, hosted_zone_id, name)?;
    Route53::change_resource_record_sets(client, delete_a_record_request).await?;
    let create_a_record_request =
        get_update_a_record_request(new_ip_address, RecordUpdateKind::CREATE, hosted_zone_id, name)?;
    Route53Client::change_resource_record_sets(client, create_a_record_request).await?;
    Ok(())
}

pub fn get_a_records_for_domains(client: &Route53Client, config: super::config::Config) -> super::config::Config {
    let new_domains = config.domains.into_iter().map(|domain| {
        let ip_addresses = match get_current_a_record(&client, &domain.hosted_zone_id) {
            Ok(addresses) => addresses,
            Err(error_message) => {
                eprintln!("{}", error_message);
                process::exit(1);
            }
        };
        super::config::Domain{ hosted_zone_id: domain.hosted_zone_id, domain_name: domain.domain_name, ip_addresses: ip_addresses }
    }).collect();
    super::config::Config{ domains: new_domains }
}