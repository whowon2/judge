use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub struct Submission {
    pub id: i32,
    pub code: String,
    pub language: String,
    pub problem_id: i32,
    pub status: String,
}

#[derive(Debug, FromRow)]
pub struct Problem {
    // pub id: i32,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub difficulty: String,
}

#[derive(Serialize)]
pub struct TestCaseResult {
    pub input: String,
    pub expected: String,
    pub actual: String,
    pub error: Option<String>, // For Runtime Errors (stderr)
    pub index: usize,
}

#[derive(Serialize)]
pub struct JudgeReport {
    pub passed: bool,
    pub total_tests: usize,
    pub passed_count: usize,
    pub failure_details: Option<TestCaseResult>, // None if all passed
}
