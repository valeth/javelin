use {
    std::{
        collections::HashSet,
        net::SocketAddr,
    },
    anyhow::Result,
    javelin_types::{
        async_trait,
        models::{UserRepository, User},
    },
    serde::Deserialize,
    javelin_core::Config,
    mongodb::{Client, bson::doc},
};


#[derive(Clone, Deserialize)]
struct DatabaseConfig {
    #[serde(default = "default_db_addr")]
    addr: SocketAddr,

    #[serde(default = "default_db_name")]
    dbname: String,

    #[serde(default)]
    password: String,

    #[serde(default)]
    username: String,
}

impl DatabaseConfig {
    pub fn uri(&self) -> String {
        let mut auth = String::new();

        if !self.username.is_empty() {
            auth += &self.username;
            if !self.password.is_empty() {
                auth += &format!(":{}", self.password)
            }
            auth += "@";
        }

        format!("mongodb://{}{}", auth, self.addr)
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            addr: default_db_addr(),
            dbname: default_db_name(),
            ..Default::default()
        }
    }
}

fn default_db_addr() -> SocketAddr {
    SocketAddr::from(([127, 0, 0, 1], 27017))
}

fn default_db_name() -> String {
    "javelin".to_string()
}


const COLLECTION_NAMES: &'static [&'static str] = &[
    "users",
];


#[derive(Clone)]
pub struct Database {
    config: DatabaseConfig,
    client: Client,
}

impl Database {
    pub async fn new(config: &Config) -> Self {
        let config: DatabaseConfig = config
            .get("database.mongo")
            .unwrap_or_default();

        let client = Client::with_uri_str(&config.uri()).await
            .expect("Failed to connect to MongoDB");

        log::info!("Connected to mongodb at {}", config.addr);

        initialize_collections(&client, &config.dbname).await
            .expect("Failed to initialize database");

        Self { config, client }
    }
}

async fn initialize_collections(client: &Client, dbname: &str) -> Result<()> {
    let db = client.database(&dbname);
    let required_collections = COLLECTION_NAMES
        .iter()
        .map(ToString::to_string)
        .collect::<HashSet<_>>();

    let available_collections = db
        .list_collection_names(None).await?
        .into_iter()
        .collect::<HashSet<_>>();

    for name in required_collections.difference(&available_collections) {
        db.create_collection(&name, None).await?;
        log::debug!("Created collection {}", name);
    }

    Ok(())
}

#[async_trait]
impl UserRepository for Database {
    async fn user_by_name(&self, name: &str) -> Option<User> {
        let db = self.client.database(&self.config.dbname);
        db.collection("users")
            .find_one(doc!{ "name": name }, None).await
            .expect("Failed to fetch user")
            .map(|doc| {
                log::debug!("Found user");
                let key = doc.get_str("key").expect("User didn't have a key");
                log::debug!("Got key {}", key);
                User { name: name.to_string(), key: key.to_string() }
            })
    }
}
