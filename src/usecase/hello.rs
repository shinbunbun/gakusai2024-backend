use crate::domain::repository::repository::HelloRepository;

pub trait HelloUsecaseTrait: Send + Sync {
    fn new(repository: Box<dyn HelloRepository>) -> Self
    where
        Self: Sized;
    fn insert(&self, hello: crate::domain::hello::Hello);
    fn find(&self, name: String) -> crate::domain::hello::Hello;
}

pub struct HelloUsecase {
    repository: Box<dyn HelloRepository>,
}

impl HelloUsecaseTrait for HelloUsecase {
    fn new(repository: Box<dyn HelloRepository>) -> Self {
        Self { repository }
    }

    fn insert(&self, hello: crate::domain::hello::Hello) {
        self.repository.insert(hello);
    }

    fn find(&self, name: String) -> crate::domain::hello::Hello {
        self.repository.find(name)
    }
}
