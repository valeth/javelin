#[cfg_attr(all(feature = "db-sqlite", not(feature = "db-mongo")), path = "database/sqlite.rs")]
#[cfg_attr(all(feature = "db-mongo", not(feature = "db-sqlite")), path = "database/mongo.rs")]
mod backend;

pub use backend::Database;
