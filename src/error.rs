use sea_orm::DbErr;
use thiserror::Error;
use tonic::Status;

#[derive(Error, Debug)]
pub enum CustomError {
    #[error("Db error: {0}")]
    Db(#[from] DbErr),
    #[error("record not found: {0}")]
    DbNotFound(String),
    #[error("Mutex error")]
    MutexError,
}

impl From<CustomError> for Status {
    fn from(val: CustomError) -> Self {
        match val {
            CustomError::Db(err) => Status::internal(err.to_string()),
            CustomError::DbNotFound(err) => Status::internal(err),
            CustomError::MutexError => Status::internal("Mutex error".to_string()),
        }
    }
}
