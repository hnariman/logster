use std::{
    env,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{anyhow, Ok, Result};
use aws_config::{
    meta::region::RegionProviderChain, profile::ProfileFileCredentialsProvider, Region,
};
use chrono::{DateTime, Utc};
use clap::Parser;

use aws_sdk_cloudwatchlogs::{self as cloudwatch};

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// The name of the CloudWatch Log Group
    // #[clap(short, long)]
    // group: String,

    /// How far back to look for logs (e.g., 1h, 30m, 2d)
    #[clap(short, long, default_value = "1h")]
    time: String,
    // /// Number of log events to retrieve (maximum)
    #[clap(short, long, default_value = "1000")]
    limit: i32,
}

/// basically constrain of the AWS Cloudwatch (have pagination, but single page is 10k limit)
const MAX_LOG_LIMIT: i32 = 10_000;

fn parse_duration(duration: &str) -> Result<i64, anyhow::Error> {
    let duration = duration.to_lowercase();
    let (num, unit) = duration.split_at(duration.len() - 1);
    let num: i64 = num.parse().map_err(|_| anyhow!("Invalid number"))?;

    match unit {
        "s" => Ok(num * 1000),
        "m" => Ok(num * 60 * 1000),
        "h" => Ok(num * 60 * 60 * 1000),
        "d" => Ok(num * 24 * 60 * 60 * 1000),
        _ => Err(anyhow!("Invalid time unit. Use s, m, h, or d")),
    }
}

fn user_input(prompt: &str) -> Result<String, anyhow::Error> {
    println!("{prompt}");

    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .map_err(|e| anyhow!("Failed to read input: {}", e))?;
    Ok(input.trim().to_string())
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    println!("Hello, Logster!");
    dotenv::dotenv().ok();

    let _args = Args::parse();

    let default_region = env::var("AWS_REGION")
        .or_else(|_| user_input("Enter AWS Profile Region:"))
        .unwrap_or_else(|_| "eu-central-1".to_string());

    let region = RegionProviderChain::default_provider().or_else(Region::new(default_region));
    let profile = env::var("AWS_PROFILE").or_else(|_| user_input("Enter AWS Profile Name:"))?;
    let group = env::var("LOG_GROUP").or_else(|_| user_input("Enter CloudWatch Log Group:"))?;

    let creds = ProfileFileCredentialsProvider::builder()
        .profile_name(&profile)
        .build();

    // let config = aws_config::from_env().region(region).load().await;
    let config = aws_config::from_env()
        .region(region)
        .credentials_provider(creds)
        .load()
        .await;

    let client = cloudwatch::Client::new(&config);
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
    let lookback_ms = parse_duration(&_args.time)?;
    let start_time = now - lookback_ms;
    let limit = if _args.limit < MAX_LOG_LIMIT {
        _args.limit
    } else {
        MAX_LOG_LIMIT
    };

    let logs = client
        .filter_log_events()
        .log_group_name(group)
        .start_time(start_time)
        .end_time(now)
        .limit(limit)
        .send()
        .await?;

    // if let Some(events) = logs.events {
    //     for evt in events {
    //         if let Some(message) = evt.message {
    //             // dbg!(message);
    //             println!("{message}");
    //         }
    //     }
    // }

    if let Some(events) = logs.events {
        for evt in events {
            if let Some(message) = evt.message() {
                if let Some(timestamp) = evt.timestamp() {
                    let datetime = DateTime::<Utc>::from_timestamp(timestamp / 1000, 0)
                        .unwrap_or_else(|| Utc::now()); // Handle potential errors

                    println!("[{}] {}", datetime.to_rfc3339(), message);
                } else {
                    println!("[No Timestamp] {}", message);
                }
            }
        }
    }

    Ok(())
}
