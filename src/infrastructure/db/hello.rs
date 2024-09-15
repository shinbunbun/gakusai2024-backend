use crate::domain::repository::repository::HelloRepository;

use super::Repository;

pub struct HelloPersistence {
    repository: Repository,
}

impl HelloRepository for HelloPersistence {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            repository: Repository::new(),
        }
    }
    fn insert(&self, hello: crate::domain::hello::Hello) {
        unimplemented!()
    }

    fn find(&self, name: String) -> crate::domain::hello::Hello {
        unimplemented!()
    }
}
