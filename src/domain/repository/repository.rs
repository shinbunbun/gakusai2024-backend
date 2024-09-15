use crate::domain::hello::Hello;

pub trait HelloRepository: Send + Sync {
    fn new() -> Self
    where
        Self: Sized;
    fn insert(&self, hello: Hello);
    fn find(&self, name: String) -> Hello;
}
