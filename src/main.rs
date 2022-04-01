#[macro_use]
extern crate dotenv_codegen;
use public_ip;
use reqwest;
use serde_json::json;
use tokio;

async fn get_ip() -> Result<String, i32> {
    let s: String = if let Some(ip) = public_ip::addr().await {
        format!("{:?}", ip)
    } else {
        std::process::exit(1);
    };
    Ok(s)
}

async fn get_domain_ip() -> Result<serde_json::Value, reqwest::Error> {
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

    let result: serde_json::Value = match serde_json::from_str(&resp.text().await?) {
        Ok(r) => r,
        Err(_) => std::process::exit(1),
    };

    let s = result["result"]["content"].clone();

    Ok(s)
}

async fn set_domain_ip(ip: &str) -> Result<reqwest::Response, reqwest::Error> {
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
    let ip: String = match get_ip().await {
        Ok(ip) => ip,
        Err(_) => std::process::exit(1),
    };

    let dom_ip: serde_json::Value = match get_domain_ip().await {
        Ok(dom_ip) => dom_ip,
        Err(_) => std::process::exit(1),
    };

    let domain_ip: String = match dom_ip.as_str() {
        Some(domain_ip) => String::from(domain_ip),
        None => std::process::exit(1),
    };

    if ip != domain_ip {
        match set_domain_ip(&ip).await {
            Ok(_) => std::process::exit(0),
            Err(_) => std::process::exit(1),
        };
    } else {
        std::process::exit(0)
    };
}
