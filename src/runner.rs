use base64::{Engine as _, engine::general_purpose};
use std::io::Write;
use std::process::{Command, Stdio};

pub struct ExecutionResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

pub async fn run_python(code: &str, input_data: &str) -> ExecutionResult {
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
        .spawn()
        .expect("Failed to spawn docker process");

    // write input to stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(input_data.as_bytes())
            .expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to read stdout");

    ExecutionResult {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        exit_code: output.status.code().unwrap_or(-1),
    }
}
