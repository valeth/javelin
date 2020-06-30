use {
    std::path::PathBuf,
    r2d2::Pool,
    r2d2_sqlite::SqliteConnectionManager,
    javelin_types::{
        async_trait,
        models::{UserRepository, User},
    },
    javelin_core::Config,
};


#[derive(Clone)]
pub struct Database {
    pool: Pool<SqliteConnectionManager>
}


impl Database {
    pub async fn new(config: &Config) -> Self {
        let path: PathBuf = config.get("database.sqlite.path").unwrap();
        let manager = SqliteConnectionManager::file(path);
        let pool = Pool::builder()
            .max_size(20)
            .build(manager)
            .unwrap();

        Self { pool }
    }
}

#[async_trait]
impl UserRepository for Database {
    async fn user_by_name(&self, name: &str) -> Option<User> {
        let conn = self.pool.get().unwrap();
        let mut stmt = conn.prepare("SELECT name, key FROM users WHERE name=?").unwrap();
        stmt.query_row(&[name], |row| {
                Ok(User {
                    name: row.get(0)?,
                    key: row.get(1)?
                })
            })
            .ok()
    }
}
