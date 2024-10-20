use anyhow::Result;
use javelin_core::Config;
use javelin_types::async_trait;
pub use javelin_types::models::{Error, User, UserRepository};
use tracing::{debug, trace};


type Pool = sqlx::SqlitePool;


#[derive(Clone)]
pub struct Database {
    pool: Pool,
}


impl Database {
    pub async fn new(config: &Config) -> Self {
        let path: String = config.get("database.sqlite.path").unwrap();
        let pool = sqlx::SqlitePool::connect(&format!("sqlite:{path}"))
            .await
            .expect("Failed to connect to database");

        Self { pool }
    }
}

#[async_trait]
impl UserRepository for Database {
    async fn user_by_name(&self, name: &str) -> Result<Option<User>, Error> {
        trace!(%name, "Querying user");
        let user = sqlx::query_as!(User, "SELECT name, key FROM users WHERE name = $1", name)
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| Error::LookupFailed)?;
        trace!(?user);
        Ok(user)
    }

    async fn add_user_with_key(&mut self, name: &str, key: &str) -> Result<(), Error> {
        let query = match self.user_by_name(name).await? {
            Some(user) if user.key == key => {
                debug!("User with key already exists, no update required");
                return Ok(());
            }
            Some(_) => {
                debug!("Updating existing user");
                sqlx::query!("UPDATE users SET key = $1 WHERE name = $2", key, name)
            }
            None => {
                debug!("Creating new user");
                sqlx::query!("INSERT INTO users (name, key) VALUES ($1, $2)", name, key)
            }
        };

        query
            .execute(&self.pool)
            .await
            .map_err(|_| Error::UpdateFailed)?;

        Ok(())
    }
}
