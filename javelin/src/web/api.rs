use {
    std::{
        error::Error as StdError,
        fmt::{self, Display},
    },
    warp::{
        Filter,
        Reply,
        Rejection,
        filters::BoxedFilter,
    },
    serde_json::json,
    crate::Shared,
};


#[derive(Clone, Debug)]
pub enum Error {
    NoSuchResource,
    StreamNotFound,
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::NoSuchResource => "No such resource",
            Error::StreamNotFound => "Stream could not be found"
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}


pub(crate) fn api(shared: Shared) -> BoxedFilter<(impl Reply,)> {
    active_streams(shared.clone())
        .or(stream_stats(shared.clone()))
        .or(server_info())
        .or_else(|err: Rejection| {
            if err.is_not_found() {
                Err(warp::reject::custom(Error::NoSuchResource))
            } else {
                Err(err)
            }
        })
        .boxed()
}

fn active_streams(shared: Shared) -> BoxedFilter<(impl Reply,)> {
    warp::path("active-streams")
        .map(move || {
            let streams = shared.streams.read();
            let active = streams.iter()
                .filter_map(|(k, v)| {
                    if v.has_publisher() {
                        Some(k.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<String>>();

            let json = json!({
                "streams": active
            });

            warp::reply::json(&json)
        })
        .boxed()
}

fn stream_stats(shared: Shared) -> BoxedFilter<(impl Reply,)> {
    warp::path("stream-stats").and(warp::path::param())
        .and_then(move |app_name: String| {
            let streams = shared.streams.read();
            match streams.get(&app_name) {
                Some(stream) => {
                    let metadata = stream.metadata.clone()
                        .map(|m| json!({
                            "video": {
                                "codec": m.video_codec,
                                "bitrate": m.video_bitrate_kbps,
                                "framerate": m.video_frame_rate,
                                "width": m.video_width,
                                "height": m.video_height
                            },
                            "audio": {
                                "codec": m.audio_codec,
                                "bitrate": m.audio_bitrate_kbps,
                                "sample_rate": m.audio_sample_rate,
                                "channels": m.audio_channels
                            }
                        }));

                    let json = json!({
                        "app_name": app_name,
                        "start_time": stream.publish_start,
                        "metadata": metadata
                    });
                    Ok(warp::reply::json(&json))
                },
                None => {
                    Err(warp::reject::custom(Error::StreamNotFound))
                }
            }
        })
        .boxed()
}

fn server_info() -> BoxedFilter<(impl Reply,)> {
    warp::path("server-info")
        .map(|| {
            let json = json!({
                "version": env!("CARGO_PKG_VERSION"),
                "authors": env!("CARGO_PKG_AUTHORS").split_terminator(':').collect::<Vec<_>>(),
                "backend": env!("CARGO_PKG_NAME"),
            });

            Ok(warp::reply::json(&json))
        })
        .boxed()
}
