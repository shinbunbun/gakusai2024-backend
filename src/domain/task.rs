use entity::task::Model;
use time::OffsetDateTime;

pub type Task = Model;

pub trait TaskExt {
    fn update(
        &self,
        title: Option<String>,
        description: Option<String>,
        user_id: Option<String>,
        due_date: Option<time::OffsetDateTime>,
        priority: Option<i32>,
        weight: Option<i32>,
    ) -> Self;
}

impl TaskExt for Task {
    fn update(
        &self,
        title: Option<String>,
        description: Option<String>,
        user_id: Option<String>,
        due_date: Option<OffsetDateTime>,
        priority: Option<i32>,
        weight: Option<i32>,
    ) -> Self {
        Self {
            id: self.id,
            title: title.unwrap_or_else(|| self.title.clone()),
            description: description.unwrap_or_else(|| self.description.clone()),
            user_id: user_id.unwrap_or_else(|| self.user_id.clone()),
            due_date: due_date.unwrap_or(self.due_date),
            priority: priority.unwrap_or(self.priority),
            weight: weight.unwrap_or(self.weight),
            created_at: self.created_at,
            updated_at: OffsetDateTime::now_utc(),
        }
    }
}
