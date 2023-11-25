use std::{env, thread, time::Duration};

use cloudflare::{
    endpoints::dns::{DnsContent, UpdateDnsRecord, UpdateDnsRecordParams},
    framework::{async_api::Client, auth::Credentials, Environment},
};

fn get_var(key: &str) -> String {
    env::var(key).expect(key)
}

#[tokio::main]
async fn main() {
    let token = get_var("CLOUDFLARE_API_TOKEN");
    let zone_id = get_var("CLOUDFLARE_ZONE_ID");
    let record_v4_id = get_var("CLOUDFLARE_RECORD_V4_ID");
    let record_v6_id = get_var("CLOUDFLARE_RECORD_V6_ID");

    let client = Client::new(
        Credentials::UserAuthToken { token },
        Default::default(),
        Environment::Production,
    )
    .expect("failed to create client");

    let mut ipv4 = None;
    let mut ipv6 = None;
    loop {
        println!("querying ipv4 address...");
        if let Some(new_ip) = public_ip::addr_v4().await {
            println!("ipv4 address: {new_ip}");

            if ipv4 != Some(new_ip) {
                ipv4 = Some(new_ip);
                if let Err(err) = update_ddns(
                    DnsContent::A { content: new_ip },
                    &client,
                    &zone_id,
                    &record_v4_id,
                )
                .await
                {
                    eprintln!("failed to update dns record: {err}");
                }
            }
        } else {
            eprintln!("unable to find ipv4 address");
        }

        println!("querying ipv6 address...");
        if let Some(new_ip) = public_ip::addr_v6().await {
            println!("ipv6 address: {new_ip}");

            if ipv6 != Some(new_ip) {
                ipv6 = Some(new_ip);
                if let Err(err) = update_ddns(
                    DnsContent::AAAA { content: new_ip },
                    &client,
                    &zone_id,
                    &record_v6_id,
                )
                .await
                {
                    eprintln!("failed to update dns record: {err}");
                }
            }
        } else {
            eprintln!("unable to find ipv6 address");
        }

        thread::sleep(Duration::from_secs(300));
    }
}

async fn update_ddns(
    record: DnsContent,
    client: &Client,
    zone_identifier: &str,
    identifier: &str,
) -> anyhow::Result<()> {
    println!("updating dns record...");

    client
        .request(&UpdateDnsRecord {
            zone_identifier,
            identifier,
            params: UpdateDnsRecordParams {
                ttl: None,
                proxied: Some(false),
                name: "home",
                content: record,
            },
        })
        .await?;

    println!("update successful");

    Ok(())
}
