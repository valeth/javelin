use {
    std::path::PathBuf,
    r2d2_sqlite::{
        SqliteConnectionManager,
        rusqlite::{params, OptionalExtension},
    },
    javelin_types::{
        async_trait,
        models::{UserRepository, User, Error},
    },
    javelin_core::Config,
};

type Pool = r2d2::Pool<SqliteConnectionManager>;

#[derive(Clone)]
pub struct Database {
    pool: Pool,
}


impl Database {
    pub async fn new(config: &Config) -> Self {
        let path: PathBuf = config.get("database.sqlite.path").unwrap();
        let manager = SqliteConnectionManager::file(path);
        let pool = Pool::builder()
            .max_size(20)
            .build(manager)
            .unwrap();

        initialize_tables(&pool);

        Self { pool }
    }
}

fn initialize_tables(pool: &Pool) {
    let conn = pool.get().unwrap();

    log::debug!("Initializing database");

    let create_users = include_str!("sqlite/users.sql");

    conn.execute(create_users, params![])
        .expect("Failed to create users table");
}

#[async_trait]
impl UserRepository for Database {
    async fn user_by_name(&self, name: &str) -> Result<Option<User>, Error> {
        let conn = self.pool.get()
            .map_err(|_| Error::LookupFailed)?;

        let mut stmt = conn
            .prepare("SELECT name, key FROM users WHERE name=?")
            .map_err(|_| Error::LookupFailed)?;

        stmt.query_row(&[name], |row| {
                Ok(User {
                    name: row.get(0)?,
                    key: row.get(1)?
                })
            })
            .optional()
            .map_err(|_| Error::LookupFailed)
    }

    async fn add_user_with_key(&mut self, name: &str, key: &str) -> Result<(), Error> {
        let conn = self.pool.get()
            .map_err(|_| Error::UpdateFailed)?;

        let mut stmt = conn
            .prepare("SELECT name, key FROM users WHERE name=?")
            .map_err(|_| Error::UpdateFailed)?;

        stmt.execute(&[name, key])
            .map_err(|_| Error::UpdateFailed)?;

        Ok(())
    }
}
