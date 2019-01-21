use std::{
    error::Error as StdError,
    fmt::{self, Display},
};
use warp::{
    Filter,
    Reply,
    filters::BoxedFilter,
};
use serde_json::json;
use crate::Shared;


#[derive(Clone, Debug)]
pub enum Error {
    NoSuchResource,
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::NoSuchResource => "No such resource"
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

pub(crate) fn api(shared: Shared) -> BoxedFilter<(impl Reply,)> {
    warp::path("active-streams")
        .map(move || {
            let json = json!({
                "streams": active_streams(&shared)
            });
            warp::reply::json(&json)
        })
        .or_else(|_err| {
            Err(warp::reject::custom(Error::NoSuchResource))
        })
        .boxed()
}

fn active_streams(shared: &Shared) -> Vec<String> {
    let streams = shared.streams.read();
    streams.iter()
        .filter_map(|(k, v)| {
            if v.has_publisher() {
                Some(k.clone())
            } else {
                None
            }
        })
        .collect()
}
