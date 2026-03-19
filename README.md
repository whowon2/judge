# ⚖️ Judge: Remote Code Execution Service

This is the worker service for the Runner platform. It processes submitted code evaluations by picking up jobs from an AWS SQS queue, running the code in an isolated Docker sandbox, and updating the submission results in the PostgreSQL database.

## 🏗️ Stack

- **Language:** Rust (Edition 2024)
- **Async Runtime:** [Tokio](https://tokio.rs/)
- **Database driver:** [sqlx](https://github.com/launchbadge/sqlx) (PostgreSQL asynchronous driver)
- **Queue:** AWS SDK for Rust (SQS)
- **Sandboxing:** Docker via `tokio::process::Command` (running `python:3.9-slim` with memory limits and no network access)

## ⚙️ How it Works

1. The Judge continuously polls the designated AWS SQS queue using long-polling.
2. Upon receiving a job containing a `submission_id`, it fetches the submitted code and problem information/test cases from the Postgres database.
3. It spawns a process to run the submitted Python code inside a low-privilege, network-less Docker container, passing the code via base64 encoded streams.
4. The code is executed against each test case's input, and `stdout` is compared with the expected output.
5. Constraints (e.g. 2s Time Limit Exceeded) and process crashes (Runtime Errors) are managed gracefully.
6. The compiled results (`PASSED`, `FAILED`, `TLE`) and traceback details are serialized into JSON and saved back to the database.

## 🚀 Getting Started

### Prerequisites

- [Rust & Cargo](https://rustup.rs/) (Stable toolchain)
- [Docker](https://www.docker.com/) installed and running on the host machine.
- PostgreSQL Database (running locally or via the root `docker-compose.yml`)
- AWS SQS Queue

### Environment Variables

Create a `.env` file inside the `judge` directory. Note that the `.env` footprint is ignored by git for security:

```env
DATABASE_URL=postgres://user:password@localhost/runner
SQS_QUEUE_URL=https://sqs.sa-east-1.amazonaws.com/your-account-id/your-queue-name
AWS_ACCESS_KEY_ID=your_aws_key
AWS_SECRET_ACCESS_KEY=your_aws_secret
AWS_REGION=sa-east-1
```

### Running Locally

```bash
cargo run
```

### Formatting

To format the rust codebase, use cargo:

```bash
cargo fmt
```
