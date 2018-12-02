use clap::{Arg, App, ArgMatches};

pub fn build_args<'a>() -> ArgMatches<'a> {
    App::new("Javelin")
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
            .takes_value(true))
        .get_matches()
}