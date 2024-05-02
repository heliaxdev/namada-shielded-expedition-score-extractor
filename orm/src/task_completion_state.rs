use diesel::{Insertable, Queryable, Selectable};
use serde::Serialize;

use crate::schema::task_completion_state;

#[derive(Serialize, Queryable, Selectable, Insertable, Clone)]
#[diesel(table_name = task_completion_state)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TaskCompletionStateDb {
    pub id: i32,
    pub last_processed_height: i32,
    pub last_processed_time: chrono::NaiveDateTime,
}

pub type TaskCompletionStateInsertDb = TaskCompletionStateDb;
