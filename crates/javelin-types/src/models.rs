use thiserror::Error;

use crate::async_trait;


#[derive(Debug, Error)]
pub enum Error {
    #[error("Database lookup failed")]
    LookupFailed,

    #[error("Database update failed")]
    UpdateFailed,
}


#[derive(Debug)]
pub struct User {
    pub name: String,
    pub key: String,
}

#[async_trait]
pub trait UserRepository {
    async fn user_by_name(&self, name: &str) -> Result<Option<User>, Error>;

    async fn add_user_with_key(&mut self, name: &str, key: &str) -> Result<(), Error>;

    async fn user_has_key(&self, name: &str, key: &str) -> Result<bool, Error> {
        if let Some(user) = self.user_by_name(name).await? {
            return Ok(&user.key == key);
        }

        Ok(false)
    }
}
