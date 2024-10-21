use thiserror::Error;
use tokio_postgres::Client;

pub struct KeyValueService {
    client: Client,
}

impl KeyValueService {
    // Constructor
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    // Method to get a key, returning Result with optional value or error
    pub async fn get_key(&self, key: String) -> Result<Option<String>, ServiceError> {
        let rows = self
            .client
            .query("SELECT value FROM kv WHERE key = $1", &[&key])
            .await?;

        // Check if a row exists and return Some(value) if it does, otherwise return None
        if let Some(row) = rows.get(0) {
            let value: String = row.get(0);
            Ok(Some(value)) // Return the value wrapped in Some
        } else {
            Ok(None) // Return None if no rows were found
        }
    }

    // Method to set a key, returning Result for error handling
    pub async fn set_key(&self, key: String, value: String) -> Result<(), ServiceError> {
        match key.as_str() {
            "foo" => Err(ServiceError::AccessError { key }),
            _ => {
                self.client
            .query(
                "INSERT INTO kv (key, value) VALUES ($1, $2) ON CONFLICT (key) DO UPDATE SET value = $2;",
                &[&key, &value],
            )
            .await?;

                Ok(()) // Return Ok(()) when successful
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
