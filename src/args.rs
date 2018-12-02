use clap::{Arg, App, ArgMatches};

pub fn build_args<'a>() -> ArgMatches<'a> {
    let mut args = App::new("Javelin")
        .version("0.1.1")
        .author("Patrick Auernig <dev.patrick.auernig@gmail.com>")
        .about("Simple RTMP streaming server")
        .arg(Arg::with_name("bind")
            .short("b")
            .long("bind")
            .value_name("ADDRESS")
            .help("Host address to bind to")
            .takes_value(true))
        .arg(Arg::with_name("port")
            .short("p")
            .long("port")
            .value_name("PORT")
            .help("Port to listen on")
            .takes_value(true));

    args = if cfg!(feature = "tls") {
        args.arg(Arg::with_name("no_tls")
             .long("no-tls")
             .help("Disables TLS support"))
        .arg(Arg::with_name("cert")
              .long("tls-cert")
              .value_name("CERTIFICATE")
              .required_unless("no_tls")
              .help("The TLS certificate to use")
              .takes_value(true))
    } else {
        args
    };

    args.get_matches()
}
