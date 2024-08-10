#[cfg(all(
    feature = "db-sqlite",
    feature = "db-mongo")
)]
compile_error!("Cannot enable multiple database backends simultaneously");


#[cfg(not(any(
    feature = "db-sqlite",
    feature = "db-mongo")
))]
compile_error!("One database backend is required");


#[cfg_attr(feature = "db-sqlite", path = "database/sqlite.rs")]
#[cfg_attr(feature = "db-mongo", path = "database/mongo.rs")]
mod backend;

pub use self::backend::Database;
