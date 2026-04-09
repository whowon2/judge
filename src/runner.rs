use crate::models::Language;
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

pub async fn run(code: &str, input_data: &str, language: Language, time_limit_secs: u64) -> ExecutionResult {
    match language {
        Language::Python => run_python(code, input_data, time_limit_secs).await,
        Language::Rust => run_rust(code, input_data, time_limit_secs).await,
        Language::C => run_not_implemented("C"),
        Language::Cpp => run_not_implemented("C++"),
        Language::Java => run_not_implemented("Java"),
    }
}

fn run_not_implemented(name: &str) -> ExecutionResult {
    ExecutionResult {
        stdout: String::new(),
        stderr: format!("Language {} is not yet implemented", name),
        exit_code: 1,
        is_timeout: false,
    }
}

pub async fn run_python(code: &str, input_data: &str, time_limit_secs: u64) -> ExecutionResult {
    let b64_code = general_purpose::STANDARD.encode(code);
    let shell_command = format!(
        "echo \"{}\" | base64 -d > script.py && python3 script.py",
        b64_code
    );
    
    run_in_docker("python:3.9-slim", &shell_command, input_data, time_limit_secs).await
}

pub async fn run_rust(code: &str, input_data: &str, time_limit_secs: u64) -> ExecutionResult {
    let b64_code = general_purpose::STANDARD.encode(code);
    let shell_command = format!(
        "echo \"{}\" | base64 -d > script.rs && rustc script.rs -o program && ./program",
        b64_code
    );
    
    run_in_docker("rust:1.80-slim", &shell_command, input_data, time_limit_secs).await
}

async fn run_in_docker(
    image: &str,
    shell_command: &str,
    input_data: &str,
    time_limit_secs: u64,
) -> ExecutionResult {
    println!("   🐳 Spawning Docker Container ({})", image);

    let mut child = Command::new("docker")
        .args(&[
            "run",
            "--rm",
            "-i",
            "--network", "none",
            "--memory", "128m",
            image,
            "sh", "-c", shell_command,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .expect("Failed to spawn docker process");

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
