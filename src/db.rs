use crate::models::{Problem, Submission};
use sqlx::{PgPool, Result};

pub struct DbClient {
    pool: PgPool,
}

impl DbClient {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPool::connect(database_url).await?;
        Ok(Self { pool })
    }

    pub async fn get_submission(&self, submission_id: i32) -> Result<Submission> {
        sqlx::query_as::<_, Submission>(
            "SELECT id, code, language::text, problem_id, status FROM submission WHERE id = $1",
        )
        .bind(submission_id)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_problem(&self, problem_id: i32) -> Result<Problem> {
        sqlx::query_as::<_, Problem>(
            "SELECT id, inputs, outputs, difficulty::text FROM problem WHERE id = $1",
        )
        .bind(problem_id)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn update_submission_result(
        &self,
        id: i32,
        status: &str,
        output: &str,
    ) -> Result<()> {
        sqlx::query("UPDATE submission SET status = $1, output = $2 WHERE id = $3")
            .bind(status)
            .bind(output)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
