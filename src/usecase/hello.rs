use std::future::Future;

use mockall::automock;

use crate::{
    domain::{hello::Hello, repository::hello::HelloRepositoryTrait},
    error::CustomError,
};

#[automock]
pub trait HelloUsecaseTrait<HR: HelloRepositoryTrait + 'static> {
    fn new(repository: Box<HR>) -> Self
    where
        Self: Sized;
    fn insert(&self, hello: Hello) -> impl Future<Output = Result<String, CustomError>> + Send;
    fn find(&self, name: String) -> impl Future<Output = Result<Hello, CustomError>> + Send;
}

pub struct HelloUsecase<HR: HelloRepositoryTrait> {
    repository: Box<HR>,
}

impl<HR: HelloRepositoryTrait + 'static> HelloUsecaseTrait<HR> for HelloUsecase<HR> {
    fn new(repository: Box<HR>) -> Self {
        Self { repository }
    }

    fn insert(&self, hello: Hello) -> impl Future<Output = Result<String, CustomError>> + Send {
        self.repository.insert(hello)
    }

    fn find(&self, name: String) -> impl Future<Output = Result<Hello, CustomError>> + Send {
        self.repository.find(name)
    }
}

#[cfg(test)]
mod tests {

    use mockall::predicate::eq;

    use super::*;
    use crate::domain::{hello::Hello, repository::hello::MockHelloRepositoryTrait};

    #[tokio::test]
    async fn test_insert() {
        let mut mock = MockHelloRepositoryTrait::default();
        mock.expect_insert()
            .returning(|_| Box::pin(async { Ok("test_name".to_string()) }));

        let usecase = HelloUsecase::new(Box::new(mock));
        let hello = Hello {
            name: "test_name".to_string(),
            message: "test_message".to_string(),
        };
        let result = usecase.insert(hello).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test_name".to_string());
    }

    #[tokio::test]
    async fn test_find() {
        let mut mock = MockHelloRepositoryTrait::default();
        let except_hello = Hello {
            name: "test_name".to_string(),
            message: "test_message".to_string(),
        };
        mock.expect_find()
            .with(eq("test_name".to_string()))
            .returning(move |_| {
                Box::pin({
                    let value = except_hello.clone();
                    async move { Ok(value) }
                })
            });

        let usecase = HelloUsecase::new(Box::new(mock));
        let result = usecase.find("test_name".to_string()).await;
        assert!(result.is_ok());
    }
}
