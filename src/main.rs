#[macro_use]
extern crate dotenv_codegen;
use public_ip;
use reqwest;
use serde::{Deserialize, Serialize};
use std::error::Error;
use tokio;
use log::info;

/// asynchronously polls services to find public IP,
/// returns first it gets back.  Uses public_ip crate
async fn get_ip() -> Result<String, Box<dyn Error>> {
    let ip: String = if let Some(ip) = public_ip::addr().await {
        format!("{:?}", ip)
    } else {
        // Safely exits if there is an error obtaining IP
        std::process::exit(1);
    };
    Ok(ip)
}

/// sends get request to cloudflare API to obtain the current
/// IP content on the domain.  Formates response with serde json
/// and returns the value.
async fn get_domain_ip() -> Result<String, Box<dyn Error>> {
    let client: reqwest::Client = reqwest::Client::new();
    let resp = client
        .get(format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
            dotenv!("ZONEID"),
            dotenv!("ID")
        ))
        .header("Authorization", dotenv!("TOKEN"))
        .header("content-Type", "application/json")
        .send()
        .await?;

    #[derive(Deserialize)]
    struct Response {
        result: Content,
    }

    #[derive(Deserialize)]
    struct Content {
        content: String,
    }

    // Creates json from the response
    let result: Response = serde_json::from_str(&resp.text().await?)?;
    let ip: String = result.result.content.clone();

    Ok(ip)
}

/// sets the cloudflare domain IP through a put
/// request after it is determined that the
/// current public IP != the current domain IP
async fn set_domain_ip(ip: &str) -> Result<reqwest::Response, Box<dyn Error>> {
    // Create struct for the data payload
    #[derive(Serialize)]
    struct Payload {
        r#type: String,
        name: String,
        content: String,
        ttl: i32,
        proxied: bool,
    }
    // Import values from .env file and assign them to struct fields
    // ttl should be auto and proxied should be false unless you
    // plan to use cloudflare tunnel to safely connect to the remote host.
    let payload: Payload = Payload {
        r#type: dotenv!("TYPE").to_string(),
        name: dotenv!("RECORD").to_string(),
        content: ip.to_string(),
        ttl: 1,
        proxied: false,
    };
    // Convert our Payload struct to json for the data payload
    let payload_json = serde_json::to_string(&payload)?;

    // Make PUT request to cloudlfare to set the updated IP
    let client: reqwest::Client = reqwest::Client::new();
    let resp = client
        .put(format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
            dotenv!("ZONEID"),
            dotenv!("ID")
        ))
        .header("Authorization", dotenv!("TOKEN"))
        .header("content-Type", "application/json")
        .json(&payload_json)
        .send()
        .await?;

    Ok(resp)
}

/// Checks if current public IP is different than current domain IP,
/// then sends PUT request to update the domain IP to the current public
/// IP if they are different or will exit safely if they are the same.
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Enable Logging
    env_logger::init();
    
    // obtains current public IP
    let ip: String = get_ip().await?;
    info!("Current public IP is {}", ip);

    // obtains current domain IP as a Result<>
    let domain_ip: String = get_domain_ip().await?;
    info!("Current Domain IP is {}", domain_ip);

    if ip != domain_ip {
        set_domain_ip(&ip).await?;
        info!("Domain IP has been updated");
    } else {
        info!("IP's match, no update necessary");
        std::process::exit(0);
    };

    Ok(())
}
