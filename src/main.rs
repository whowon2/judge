use std::{env, time::Duration};

use aws_config::{BehaviorVersion, meta::region::RegionProviderChain};
use aws_sdk_sqs::Client;
use serde::Deserialize;
use tokio::time::sleep;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Job {
    question_letter: String,
    submission_id: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // AWS Connection
    let region_provider = RegionProviderChain::default_provider().or_else("sa-east-1");
    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(region_provider)
        .load()
        .await;
    let client = Client::new(&config);
    let queue_url = env::var("SQS_QUEUE_URL").expect("SQS_QUEUE_URL must be set");

    println!("Worker started. Polling queue {}", queue_url);

    loop {
        let rcv_result = client
            .receive_message()
            .queue_url(&queue_url)
            .max_number_of_messages(1)
            .wait_time_seconds(5)
            .send()
            .await;

        match rcv_result {
            Ok(response) => {
                if let Some(messages) = response.messages {
                    for msg in messages {
                        if let Some(body) = &msg.body {
                            match serde_json::from_str::<Job>(body) {
                                Ok(job) => {
                                    println!("Received Job: {:?}", job);

                                    process_job(&job).await;
                                }
                                Err(e) => eprintln!("Failed to parse JSON: {}", e),
                            }
                        }
                    }
                }
            }
            Err(e) => println!("AWS SQS Connection Failed: {}", e),
        }
    }
}

async fn process_job(job: &Job) {
    println!("\tCreating sandbox for submission {}...", job.submission_id);
    sleep(Duration::from_secs(2)).await;
    println!("\tJudging answer to question {}", job.question_letter);
    println!("\tExecution complete.");
}
