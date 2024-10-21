use tokio_postgres::Client;
use tokio_postgres::Error;
use tokio_postgres::NoTls;
pub struct Repository {
    client: Client,
}

impl Repository {
    // Spins up a new repository and performs DB migration
    pub async fn new() -> Self {
        // Connect to the database.
        let (client, connection) = tokio_postgres::connect(
            "host=localhost user=postgres password=mysecretpassword port=5432",
            NoTls,
        )
        .await
        .unwrap();

        // The connection object performs the actual communication with the database,
        // so spawn it off to run on its own.
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        client
            .batch_execute(
                "
CREATE TABLE kv (
    key TEXT PRIMARY KEY,
    value TEXT
);
",
            )
            .await
            .ok();

        Self { client }
    }

    // Method to get a key, returning Result with optional value or error
    pub async fn get_key(&self, key: String) -> Result<Vec<String>, Error> {
        let rows = self
            .client
            .query("SELECT value FROM kv WHERE key = $1", &[&key])
            .await?;

        let values = rows
            .iter() // Create an iterator over references
            .map(|r| r.get::<usize, String>(0)) // Use map to transform each number
            .collect();

        Ok(values)
    }

    pub async fn set_key(&self, key: String, value: String) -> Result<(), Error> {
        self.client
            .query(
                "INSERT INTO kv (key, value) VALUES ($1, $2) ON CONFLICT (key) DO UPDATE SET value = $2;",
                &[&key, &value],
            )
            .await?;

        Ok(())
    }
}
