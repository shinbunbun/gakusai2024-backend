use std::future::Future;

use mockall::automock;
use uuid::Uuid;

use crate::{
    domain::{repository::task::TaskRepositoryTrait, task::Task},
    error::CustomError,
};

#[automock]
pub trait TaskUsecaseTrait<TR: TaskRepositoryTrait + 'static> {
    fn new(repository: Box<TR>) -> Self
    where
        Self: Sized;
    fn insert(&self, task: Task) -> impl Future<Output = Result<Uuid, CustomError>> + Send;
    fn find(&self, id: Uuid) -> impl Future<Output = Result<Task, CustomError>> + Send;
    fn find_from_userid(&self, user_id: String) -> impl Future<Output = Result<Vec<Task>, CustomError>> + Send;
}

pub struct TaskUsecase<TR: TaskRepositoryTrait> {
    repository: Box<TR>,
}

impl<TR: TaskRepositoryTrait + 'static> TaskUsecaseTrait<TR> for TaskUsecase<TR> {
    fn new(repository: Box<TR>) -> Self {
        Self { repository }
    }

    fn insert(&self, task: Task) -> impl Future<Output = Result<Uuid, CustomError>> + Send {
        self.repository.insert(task)
    }

    fn find(&self, id: Uuid) -> impl Future<Output = Result<Task, CustomError>> + Send {
        self.repository.find(id)
    }

    fn find_from_userid(&self,user_id:String) -> impl Future<Output = Result<Vec<Task>,CustomError>> +Send {
        self.repository.find_from_userid(user_id)
    }
}

#[cfg(test)]
mod tests {

    use mockall::predicate::eq;
    use time::OffsetDateTime;
    use uuid::uuid;

    use super::*;
    use crate::domain::{repository::task::MockTaskRepositoryTrait, task::Task};

    #[tokio::test]
    async fn test_task_insert() {
        let test_uuid = uuid!("00000000-0000-0000-0000-ffff00000000");

        let mut mock = MockTaskRepositoryTrait::default();
        mock.expect_insert()
            .returning(|_| Box::pin(async { Ok(uuid!("00000000-0000-0000-0000-ffff00000000")) }));

        let usecase = TaskUsecase::new(Box::new(mock));
        let task = Task {
            id: test_uuid,
            title: "test_title".to_string(),
            description: "test_description".to_string(),
            due_date: OffsetDateTime::now_utc(),
            priority: 1,
            weight: 1,
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            user_id: "testuserid".to_string(),
        };
        let result = usecase.insert(task).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), test_uuid);
    }

    #[tokio::test]
    async fn test_task_find() {
        let test_uuid = uuid!("00000000-0000-0000-0000-ffff00000000");

        let mut mock = MockTaskRepositoryTrait::default();
        let except_task = Task {
            id: test_uuid,
            title: "test_title".to_string(),
            description: "test_description".to_string(),
            due_date: OffsetDateTime::now_utc(),
            priority: 1,
            weight: 1,
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            user_id: "testuserid".to_string(),
        };
        mock.expect_find()
            .with(eq(uuid!("00000000-0000-0000-0000-ffff00000000")))
            .returning(move |_| {
                Box::pin({
                    let value = except_task.clone();
                    async move { Ok(value) }
                })
            });

        let usecase = TaskUsecase::new(Box::new(mock));
        let result = usecase
            .find(uuid!("00000000-0000-0000-0000-ffff00000000"))
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_listtasks_find() {
        let test_uuid = uuid!("00000000-0000-0000-0000-ffff00000000");
        let test_uuid2 = uuid!("00000000-0000-0000-0000-ffff00000001");

        let mut mock = MockTaskRepositoryTrait::default();
        let except_tasks = vec![
            Task {
                id: test_uuid,
                title: "test_title".to_string(),
                description: "test_description".to_string(),
                due_date: OffsetDateTime::now_utc(),
                priority: 1,
                weight: 1,
                created_at: OffsetDateTime::now_utc(),
                updated_at: OffsetDateTime::now_utc(),
                user_id: "harukun".to_string(),
            },
            Task {
                id: test_uuid2,
                title: "test_title".to_string(),
                description: "test_description".to_string(),
                due_date: OffsetDateTime::now_utc(),
                priority: 1,
                weight: 1,
                created_at: OffsetDateTime::now_utc(),
                updated_at: OffsetDateTime::now_utc(),
                user_id: "harukun".to_string(),
            }
        ];
        mock.expect_find_from_userid()
            .with(eq("harukun".to_string()))
            .returning(move |_| {
                Box::pin({
                    let value = except_tasks.clone();
                    async move { Ok(value) }
                })
            });

        let usecase = TaskUsecase::new(Box::new(mock));
        let result = usecase
            .find_from_userid("harukun".to_string())
            .await;
        assert!(result.is_ok());
    }
}
