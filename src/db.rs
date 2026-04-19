use crate::models::{Problem, Submission, SubmissionStatus};
use sqlx::{PgPool, Result};
use uuid::Uuid;

pub struct DbClient {
    pool: PgPool,
}

impl DbClient {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPool::connect(database_url).await?;
        Ok(Self { pool })
    }

    pub async fn get_next_submission(&self) -> Result<Option<Submission>> {
        sqlx::query_as::<_, Submission>(
            "UPDATE submission 
             SET status = 'RUNNING' 
             WHERE id = (
                 SELECT id FROM submission 
                 WHERE status = 'PENDING' 
                 ORDER BY created_at ASC 
                 FOR UPDATE SKIP LOCKED 
                 LIMIT 1
             ) 
             RETURNING id, code, language, problem_id, user_id, contest_id, question_letter",
        )
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn get_problem(&self, problem_id: Uuid) -> Result<Problem> {
        sqlx::query_as::<_, Problem>(
            "SELECT inputs, outputs FROM problem WHERE id = $1",
        )
        .bind(problem_id)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn update_submission_result(
        &self,
        submission: &Submission,
        status: SubmissionStatus,
        output: &str,
    ) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // 1. Update the submission record
        sqlx::query("UPDATE submission SET status = $1, output = $2 WHERE id = $3")
            .bind(status)
            .bind(output)
            .bind(submission.id)
            .execute(&mut *tx)
            .await?;

        // 2. If the submission PASSED, update the user's leaderboard entry for this contest
        if status == SubmissionStatus::PASSED {
            sqlx::query(
                "UPDATE user_on_contest 
                 SET answered = array_append(answered, $1) 
                 WHERE user_id = $2 AND contest_id = $3 
                 AND NOT ($1 = ANY(answered))",
            )
            .bind(&submission.question_letter)
            .bind(&submission.user_id)
            .bind(submission.contest_id)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }
}
