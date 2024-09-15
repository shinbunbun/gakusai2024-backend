use std::future::Future;

use crate::domain::repository::repository::HelloRepository;

pub trait HelloUsecaseTrait<HR: HelloRepository> {
    fn new(repository: Box<HR>) -> Self
    where
        Self: Sized;
    fn insert(
        &self,
        hello: crate::domain::hello::Hello,
    ) -> impl std::future::Future<Output = ()> + Send;
    fn find(&self, name: String) -> impl Future<Output = crate::domain::hello::Hello> + Send;
}

pub struct HelloUsecase<HR: HelloRepository> {
    repository: Box<HR>,
}

impl<HR: HelloRepository> HelloUsecaseTrait<HR> for HelloUsecase<HR> {
    fn new(repository: Box<HR>) -> Self {
        Self { repository }
    }

    fn insert(
        &self,
        hello: crate::domain::hello::Hello,
    ) -> impl std::future::Future<Output = ()> + Send {
        self.repository.insert(hello)
    }

    fn find(&self, name: String) -> impl Future<Output = crate::domain::hello::Hello> + Send {
        self.repository.find(name)
    }
}
