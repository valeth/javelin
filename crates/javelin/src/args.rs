use clap::{
    Arg, App, ArgMatches, AppSettings, SubCommand,
    crate_name,
    crate_version,
    crate_authors,
    crate_description,
};

#[allow(deprecated)] // clap issue: https://github.com/clap-rs/clap/issues/1552
pub fn build<'a>() -> ArgMatches<'a> {
    App::new(capitalize(crate_name!()))
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about(crate_description!())
        .setting(AppSettings::ColoredHelp)
        .arg(Arg::with_name("config_dir")
            .default_value("./")
            .short("c")
            .long("config-dir")
            .value_name("PATH")
            .help("The directory where all config files are located"))
        .subcommand(SubCommand::with_name("run"))
        .subcommand(SubCommand::with_name("permit-stream")
            .arg(Arg::with_name("user").required(true))
            .arg(Arg::with_name("key").required(true))
        ).get_matches()
}

fn capitalize(string: &str) -> String {
    string
        .chars()
        .enumerate()
        .map(|(i, c)| {
            match i {
                0 => c.to_uppercase().to_string(),
                _ => c.to_string(),
            }
        })
        .collect()
}
