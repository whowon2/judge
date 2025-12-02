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
    pub id: i32,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub difficulty: String,
}
