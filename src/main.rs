#[macro_use]
extern crate dotenv_codegen;
use public_ip;
use reqwest;
use serde_json::json;
use tokio;

async fn get_ip() -> Result<String, i32> {
    /// asynchronously polls services to find public IP,
    /// returns first it gets back.  Uses public_ip crate
    let s: String = if let Some(ip) = public_ip::addr().await {
        format!("{:?}", ip)
    } else {
        // Safely exits if there is an error obtaining IP
        std::process::exit(1);
    };
    Ok(s)
}

async fn get_domain_ip() -> Result<serde_json::Value, reqwest::Error> {
    /// sends get request to cloudflare API to obtain the current
    /// IP content on the domain.  Formates response with serde json
    /// and returns the value.
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
    // Creates json from the response
    let result: serde_json::Value = match serde_json::from_str(&resp.text().await?) {
        Ok(r) => r,
        // Safely exits if there is an error creating json
        Err(_) => std::process::exit(1),
    };

    let s = result["result"]["content"].clone();

    Ok(s)
}

async fn set_domain_ip(ip: &str) -> Result<reqwest::Response, reqwest::Error> {
    /// sets the cloudflare domain IP through a put
    /// request after it is determined that the
    /// current public IP != the current domain IP
    // Creates json for the payload that is sent in the put request
    let data = json!({
            "type": dotenv!("TYPE"),
            "name": dotenv!("RECORD"),
            "content": ip,
            "ttl": 1,
            "proxied": true});

    let client: reqwest::Client = reqwest::Client::new();
    let resp = client
        .put(format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
            dotenv!("ZONEID"),
            dotenv!("ID")
        ))
        .header("Authorization", dotenv!("TOKEN"))
        .header("content-Type", "application/json")
        .json(&data)
        .send()
        .await?;

    Ok(resp)
}

#[tokio::main]
async fn main() {
    // obtains current public IP
    let ip: String = match get_ip().await {
        Ok(ip) => ip,
        Err(_) => std::process::exit(1),
    };
    // obtains current domain IP as a Result<>
    let dom_ip: serde_json::Value = match get_domain_ip().await {
        Ok(dom_ip) => dom_ip,
        Err(_) => std::process::exit(1),
    };
    // converts the current domain IP to a String
    let domain_ip: String = match dom_ip.as_str() {
        Some(domain_ip) => String::from(domain_ip),
        None => std::process::exit(1),
    };
    /// Checks if curretn public IP is different than current domain IP,
    /// then sends put request to update the domain IP to the current
    /// public IP if they are different and then exits safely, or will
    /// exit safely if they are the same.  Exits safely upon error.
    if ip != domain_ip {
        match set_domain_ip(&ip).await {
            Ok(_) => std::process::exit(0),
            Err(_) => std::process::exit(1),
        };
    } else {
        std::process::exit(0)
    };
}
