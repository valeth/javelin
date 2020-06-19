use clap::{
    Arg, App, ArgMatches, AppSettings,
    crate_name,
    crate_version,
    crate_authors,
    crate_description,
};

#[allow(deprecated)] // clap issue: https://github.com/clap-rs/clap/issues/1552
pub fn build_args<'a>() -> ArgMatches<'a> {
    let mut app = App::new(capitalize(crate_name!()))
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about(crate_description!())
        .setting(AppSettings::ColoredHelp)
        .arg(Arg::with_name("bind")
            .short("b")
            .long("rtmp-bind")
            .alias("bind")
            .value_name("ADDRESS")
            .default_value("0.0.0.0")
            .display_order(1)
            .help("Host address to bind to"))
        .arg(Arg::with_name("port")
            .short("p")
            .long("rtmp-port")
            .alias("port")
            .value_name("PORT")
            .default_value("1935")
            .display_order(1)
            .help("Port to listen on"))
        .arg(Arg::with_name("permitted_stream_keys")
            .short("k")
            .long("permit-stream-key")
            .value_name("KEY")
            .display_order(2)
            .help("Permit a stream key for publishing")
            .multiple(true))
        .arg(Arg::with_name("republish_action")
            .long("republish-action")
            .possible_values(&["replace", "deny"])
            .default_value("replace")
            .help("The action to take when a republishing to the same application"))
        .arg(Arg::with_name("config_dir")
            .short("c")
            .long("config-dir")
            .value_name("PATH")
            .help("The directory where all config files are located"));

    let mut args = Vec::new();

    if cfg!(feature = "web") {
        args.push(Arg::with_name("http_disabled")
            .long("disable-http")
            .help("Disables the integrated web server"));

        args.push(Arg::with_name("http_bind")
            .long("http-bind")
            .value_name("ADDRESS")
            .default_value("0.0.0.0")
            .display_order(10)
            .help("The web server address"));

        args.push(Arg::with_name("http_port")
            .long("http-port")
            .value_name("PORT")
            .default_value("8080")
            .display_order(10)
            .help("The web server listening port"));
    }

    if cfg!(feature = "hls") {
        args.push(Arg::with_name("hls_disabled")
            .long("disable-hls")
            .help("Disables HLS support"));

        args.push(Arg::with_name("hls_root")
            .long("hls-root")
            .value_name("PATH")
            .display_order(20)
            .help("The directory where stream output will be placed"));
    }

    if cfg!(feature = "tls") {
        args.push(Arg::with_name("tls_enabled")
            .long("enable-tls")
            .requires("tls_cert")
            .help("Enables TLS support"));

        args.push(Arg::with_name("tls_cert")
            .long("tls-cert")
            .value_name("CERTIFICATE")
            .display_order(30)
            .help("The TLS certificate to use"));
    }

    app = app.args(&args);

    app.get_matches()
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
