use crate::repository::Repository;
use thiserror::Error;

pub struct KeyValueService {
    repo: Repository,
}

impl KeyValueService {
    pub fn new(repo: Repository) -> Self {
        Self { repo }
    }

    pub async fn get_key(&self, key: String) -> Result<Option<String>, ServiceError> {
        let mut values = self.repo.get_key(key).await?;

        let value = values.pop();
        Ok(value)
    }

    pub async fn set_key(&self, key: String, value: String) -> Result<(), ServiceError> {
        match key.as_str() {
            "foo" => Err(ServiceError::AccessError { key }),
            _ => {
                self.repo.set_key(key, value).await?;

                Ok(())
            }
        }
    }
}

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Database error)")]
    DBError(#[from] tokio_postgres::Error), // Convert tokio_postgres errors to DBError
    #[error("The is key not allowed {key}")]
    AccessError { key: String },
}
