use base64::{Engine as _, engine::general_purpose};
use std::process::Stdio;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::time::timeout;

pub struct ExecutionResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub is_timeout: bool,
}

pub async fn run_python(code: &str, input_data: &str, time_limit_secs: u64) -> ExecutionResult {
    println!("   🐳 Spawning Docker Container...");

    let b64_code = general_purpose::STANDARD.encode(code);
    // decode de script and pipe the code from rust to docker process
    let shell_command = format!(
        "echo \"{}\" | base64 -d > script.py && python3 script.py",
        b64_code
    );

    let mut child = Command::new("docker")
        .args(&[
            "run",
            "--rm",
            "-i", // keep stdin open
            "--network",
            "none", // no internet
            "--memory",
            "128m",
            "python:3.9-slim",
            "sh",
            "-c",
            &shell_command,
        ])
        .stdin(Stdio::piped()) // write
        .stdout(Stdio::piped()) // read
        .stderr(Stdio::piped()) // read
        .kill_on_drop(true)
        .spawn()
        .expect("Failed to spawn docker process");

    // write input to stdin
    if let Some(mut stdin) = child.stdin.take() {
        let input_bytes = input_data.as_bytes().to_vec();

        tokio::spawn(async move {
            stdin.write_all(&input_bytes).await.ok();
        });
    }

    let duration = Duration::from_secs(time_limit_secs);

    match timeout(duration, child.wait_with_output()).await {
        Ok(run_result) => {
            let output = run_result.expect("Failed to read stdout");

            ExecutionResult {
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                exit_code: output.status.code().unwrap_or(-1),
                is_timeout: false,
            }
        }
        Err(_) => {
            println!("\t⏳ Time Limit Exceeded! Killing container...");

            ExecutionResult {
                stdout: String::new(),
                stderr: "Time Limit Exceeded".to_string(),
                exit_code: 124,
                is_timeout: true,
            }
        }
    }
}
