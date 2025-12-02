mod db;
mod models;
mod runner;

use std::env;

use aws_config::{BehaviorVersion, meta::region::RegionProviderChain};
use aws_sdk_sqs::Client;
use serde::Deserialize;

use crate::db::DbClient;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Job {
    submission_id: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // DB
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    println!("Connecting to database...");
    let db = DbClient::new(&database_url)
        .await
        .expect("Failed to connect to DB");
    println!("Database connected");

    // AWS Connection
    let region_provider = RegionProviderChain::default_provider().or_else("sa-east-1");
    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(region_provider)
        .load()
        .await;
    let client = Client::new(&config);
    let queue_url = env::var("SQS_QUEUE_URL").expect("SQS_QUEUE_URL must be set");

    println!("Worker pulling {}", queue_url);

    loop {
        let rcv_result = client
            .receive_message()
            .queue_url(&queue_url)
            .max_number_of_messages(1)
            .wait_time_seconds(20)
            .send()
            .await;

        if let Ok(response) = rcv_result {
            if let Some(messages) = response.messages {
                for msg in messages {
                    if let Some(body) = &msg.body {
                        if let Ok(job) = serde_json::from_str::<Job>(body) {
                            println!("Received Job: {:?}", job.submission_id);

                            let sub_id_int: i32 = job.submission_id.parse().unwrap_or(0);

                            process_job(&db, sub_id_int).await;

                            // Delete msg
                            if let Some(receipt_handle) = msg.receipt_handle {
                                let _ = client
                                    .delete_message()
                                    .queue_url(&queue_url)
                                    .receipt_handle(receipt_handle)
                                    .send()
                                    .await;
                            }
                        }
                    }
                }
            }
        }
    }
}

async fn process_job(db: &DbClient, submission_id: i32) {
    let sub = match db.get_submission(submission_id).await {
        Ok(submission) => submission,
        Err(err) => {
            eprintln!("Failed to fetch submission {}: {}", submission_id, err);
            return;
        }
    };

    let problem = match db.get_problem(sub.problem_id).await {
        Ok(problem) => problem,
        Err(err) => {
            eprintln!("Failed to fetch problem: {}", err);
            return;
        }
    };

    println!(
        "\tJudging Submission {} (Language: {})",
        sub.id, sub.language
    );

    let mut all_passed = true;
    let mut final_output_for_db = String::new();

    for (i, input) in problem.inputs.iter().enumerate() {
        let expected = &problem.outputs[i];

        let result = runner::run_python(&sub.code, input).await;
        let actual = result.stdout.trim().to_string();

        if result.exit_code != 0 {
            all_passed = false;
            final_output_for_db = result.stderr;
            println!("\t❌ Runtime Error on Test {}", i + 1);
            break;
        }
        // Check for Wrong Answer
        else if actual != expected.trim() {
            all_passed = false;
            final_output_for_db = actual;
            println!("\t❌ Wrong Answer on Test {}", i + 1);
            break;
        }

        // If it's the last test and everything passed, save the last output
        if i == problem.inputs.len() - 1 {
            final_output_for_db = actual;
        }
    }

    let status = if all_passed { "PASSED" } else { "FAILED" };

    if final_output_for_db.len() > 1000 {
        final_output_for_db.truncate(1000);
        final_output_for_db.push_str("\n...[Output Truncated]");
    }

    if let Err(e) = db
        .update_submission_result(submission_id, status, &final_output_for_db)
        .await
    {
        eprintln!("\tFailed to update status: {}", e);
    } else {
        println!("\tUpdate status to: {}", status)
    }
}
