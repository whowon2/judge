mod runner;

use std::env;

use aws_config::{BehaviorVersion, meta::region::RegionProviderChain};
use aws_sdk_sqs::Client;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Job {
    // question_letter: String,
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
                                    println!("Received Job: {:?}", job.submission_id);

                                    process_job(&job).await;

                                    if let Some(receipt_handle) = msg.receipt_handle {
                                        let _ = client
                                            .delete_message()
                                            .queue_url(&queue_url)
                                            .receipt_handle(receipt_handle)
                                            .send()
                                            .await;
                                    }
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

async fn process_job(_job: &Job) {
    let user_code = r#"
import sys

# Read from Stdin
line = sys.stdin.read()
if not line:
    sys.exit(0)

parts = line.split()
a = int(parts[0])
b = int(parts[1])

print(a + b)
"#;

    let input_case = "10 5";
    let expected_output = "15";

    println!("\t🧪 Running Test Case: Input '{}'", input_case);

    let result = runner::run_python(user_code, input_case).await;

    let actual_output = result.stdout.trim();

    println!("\t---------------------------");
    if result.exit_code != 0 {
        println!("\t❌ RUNTIME ERROR");
        println!("\tError: {}", result.stderr);
    } else if actual_output == expected_output {
        println!("\t✅ PASSED");
        println!("\tOutput: {}", actual_output);
    } else {
        println!("\t❌ WRONG ANSWER");
        println!("\tExpected: {}", expected_output);
        println!("\tActual:   {}", actual_output);
    }
    println!("\t---------------------------");
}
