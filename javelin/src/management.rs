use {
    clap::ArgMatches,
    anyhow::Result,
    javelin_core::config::Config,
    javelin_types::models::UserRepository,
    crate::database::Database,
};


pub async fn permit_stream(args: &ArgMatches<'_>, config: &Config) -> Result<()> {
    let mut database_handle = Database::new(&config).await;

    let user = args.value_of("user").unwrap(); // required parameter
    let key = args.value_of("key").unwrap();  // required parameter

    database_handle.add_user_with_key(user, key).await?;

    Ok(())
}
