use anyhow::Result;
use javelin_core::config::Config;
use javelin_types::models::UserRepository;

use crate::database::Database;


pub async fn permit_stream(user: &str, key: &str, config: &Config) -> Result<()> {
    let mut database_handle = Database::new(config).await;

    database_handle.add_user_with_key(user, key).await?;

    Ok(())
}
