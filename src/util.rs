
use std::io;
use std::fmt;
use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

use iron::status;
use iron::{IronError};
use iron::headers::{Range, ByteRangeSpec};
use url::percent_encoding::{utf8_percent_encode, PATH_SEGMENT_ENCODE_SET};
use chrono::{DateTime, Local, TimeZone};

#[derive(Debug)]
pub struct StringError(pub String);

impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl Error for StringError {
    fn description(&self) -> &str {
        &self.0
    }
}

pub fn enable_string(value: bool) -> String {
    (if value { "enabled" } else { "disabled" }).to_owned()
}

pub fn encode_link_path(path: &Vec<String>) -> String {
    path.iter().map(|s| {
        utf8_percent_encode(s, PATH_SEGMENT_ENCODE_SET).to_string()
    }).collect::<Vec<String>>().join("/")
}

pub fn error_io2iron(err: io::Error) -> IronError {
    let status = match err.kind() {
        io::ErrorKind::PermissionDenied => status::Forbidden,
        io::ErrorKind::NotFound => status::NotFound,
        _ => status::InternalServerError
    };
    IronError::new(err, status)
}

#[allow(dead_code)]
pub fn parse_range(ranges: &Vec<ByteRangeSpec>, total: u64)
                   -> Result<Option<(u64, u64)>, IronError> {
    if let Some(range) = ranges.get(0) {
        let (offset, length) = match range {
            &ByteRangeSpec::FromTo(x, mut y) => { // "x-y"
                if x >= total || x > y {
                    return Err(IronError::new(
                        StringError(format!("Invalid range(x={}, y={})", x, y)),
                        status::RangeNotSatisfiable
                    ));
                }
                if y >= total {
                    y = total - 1;
                }
                (x, y - x + 1)
            }
            &ByteRangeSpec::AllFrom(x) => { // "x-"
                if x >= total {
                    return Err(IronError::new(
                        StringError(format!(
                            "Range::AllFrom to large (x={}), Content-Length: {})",
                            x, total)),
                        status::RangeNotSatisfiable
                    ));
                }
                (x, total - x)
            }
            &ByteRangeSpec::Last(mut x) => { // "-x"
                if x > total {
                    x = total;
                }
                (total - x, x)
            }
        };
        Ok(Some((offset, length)))
    } else {
        return Err(IronError::new(
            StringError("Empty range set".to_owned()),
            status::RangeNotSatisfiable
        ));
    }
}

pub fn now_string() -> String {
    Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn system_time_to_date_time(t: SystemTime) -> DateTime<Local> {
    let (sec, nsec) = match t.duration_since(UNIX_EPOCH) {
        Ok(dur) => (dur.as_secs() as i64, dur.subsec_nanos()),
        Err(e) => { // unlikely but should be handled
            let dur = e.duration();
            let (sec, nsec) = (dur.as_secs() as i64, dur.subsec_nanos());
            if nsec == 0 {
                (-sec, 0)
            } else {
                (-sec - 1, 1_000_000_000 - nsec)
            }
        },
    };
    Local.timestamp(sec, nsec)
}