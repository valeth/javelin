use clap::{
    Arg, App, ArgMatches,
    crate_name,
    crate_version,
    crate_authors,
    crate_description,
};


pub fn build_args<'a>() -> ArgMatches<'a> {
    let mut args = App::new(capitalize(crate_name!()))
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about(crate_description!())
        .arg(Arg::with_name("bind")
            .short("b")
            .long("rtmp-bind")
            .alias("bind")
            .value_name("ADDRESS")
            .help("Host address to bind to")
            .takes_value(true))
        .arg(Arg::with_name("port")
            .short("p")
            .long("rtmp-port")
            .alias("port")
            .value_name("PORT")
            .help("Port to listen on")
            .takes_value(true))
        .arg(Arg::with_name("permitted_stream_keys")
            .short("k")
            .long("permit-stream-key")
            .value_name("KEY")
            .help("Permit a stream key for publishing")
            .multiple(true)
            .takes_value(true));

    if cfg!(feature = "tls") {
        args = args
        .arg(Arg::with_name("tls_enabled")
             .long("enable-tls")
             .requires("tls_cert")
             .help("Enables TLS support"))
        .arg(Arg::with_name("tls_cert")
              .long("tls-cert")
              .value_name("CERTIFICATE")
              .help("The TLS certificate to use")
              .takes_value(true))
    }

    if cfg!(feature = "hls") {
        args = args
        .arg(Arg::with_name("hls_root")
            .long("hls-root")
            .value_name("PATH")
            .help("The directory where stream output will be placed")
            .takes_value(true))
    }

    if cfg!(feature = "web") {
        args = args
        .arg(Arg::with_name("http_bind")
            .long("http-bind")
            .value_name("ADDRESS")
            .help("The web server address")
            .takes_value(true))
        .arg(Arg::with_name("http_port")
            .long("http-port")
            .value_name("PORT")
            .help("The web server listening port")
            .takes_value(true))
        .arg(Arg::with_name("http_disabled")
            .long("disable-http")
            .help("Disables the integrated web server"))
    }

    args.get_matches()
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
