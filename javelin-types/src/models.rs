use crate::async_trait;

pub struct User {
    pub name: String,
    pub key: String,
}

#[async_trait]
pub trait UserRepository {
    async fn user_by_name(&self, name: &str) -> Option<User>;

    async fn user_has_key(&self, name: &str, key: &str) -> bool {
        if let Some(user) = self.user_by_name(name).await {
            return &user.key == key
        }

        false
    }
}
