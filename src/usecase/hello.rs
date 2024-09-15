use std::future::Future;

use crate::{
    domain::{hello::Hello, repository::hello::HelloRepository},
    error::CustomError,
};

pub trait HelloUsecaseTrait<HR: HelloRepository> {
    fn new(repository: Box<HR>) -> Self
    where
        Self: Sized;
    fn insert(&self, hello: Hello) -> impl Future<Output = Result<String, CustomError>> + Send;
    fn find(&self, name: String) -> impl Future<Output = Result<Hello, CustomError>> + Send;
}

pub struct HelloUsecase<HR: HelloRepository> {
    repository: Box<HR>,
}

impl<HR: HelloRepository> HelloUsecaseTrait<HR> for HelloUsecase<HR> {
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
