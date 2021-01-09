use hyper::header::{HeaderMap, HeaderValue, ToStrError};
use std::{
    convert::TryFrom,
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
    num::{ParseFloatError, ParseIntError},
    str::ParseBoolError,
};

#[derive(Debug)]
#[non_exhaustive]
pub enum HeaderParseError {
    NoHeaders,
    HeaderMissing {
        name: &'static str,
    },
    HeaderNotUtf8 {
        name: &'static str,
        source: ToStrError,
        value: Vec<u8>,
    },
    ParsingBoolText {
        name: &'static str,
        source: ParseBoolError,
        text: String,
    },
    ParsingFloatText {
        name: &'static str,
        source: ParseFloatError,
        text: String,
    },
    ParsingIntText {
        name: &'static str,
        source: ParseIntError,
        text: String,
    },
}

impl Display for HeaderParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::NoHeaders => f.write_str("No headers are present"),
            Self::HeaderMissing { name } => {
                write!(f, "At least one header, {:?}, is missing", name)
            }
            Self::HeaderNotUtf8 { name, value, .. } => {
                write!(f, "The header {:?} has invalid UTF-16: {:?}", name, value)
            }
            Self::ParsingBoolText { name, text, .. } => write!(
                f,
                "The header {:?} should be a bool but isn't: {:?}",
                name, text
            ),
            Self::ParsingFloatText { name, text, .. } => write!(
                f,
                "The header {:?} should be a float but isn't: {:?}",
                name, text
            ),
            Self::ParsingIntText { name, text, .. } => write!(
                f,
                "The header {:?} should be an integer but isn't: {:?}",
                name, text
            ),
        }
    }
}

impl StdError for HeaderParseError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::HeaderNotUtf8 { source, .. } => Some(source),
            Self::ParsingBoolText { source, .. } => Some(source),
            Self::ParsingFloatText { source, .. } => Some(source),
            Self::ParsingIntText { source, .. } => Some(source),
            Self::NoHeaders | Self::HeaderMissing { .. } => None,
        }
    }
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Headers {
    GlobalLimited {
        reset_after: u64,
    },
    None,
    Present {
        bucket: Option<String>,
        global: bool,
        limit: u64,
        remaining: u64,
        // when the bucket resets in unix ms
        reset: u64,
        // how long until it resets in ms
        reset_after: u64,
    },
}

impl Headers {
    pub fn is_global(&self) -> bool {
        match self {
            Self::GlobalLimited { .. } => true,
            Self::None => false,
            Self::Present { global, .. } => *global,
        }
    }

    pub fn is_none(&self) -> bool {
        matches!(self, Headers::None)
    }

    pub fn is_present(&self) -> bool {
        matches!(self, Headers::Present { .. })
    }
}

impl TryFrom<&'_ HeaderMap<HeaderValue>> for Headers {
    type Error = HeaderParseError;

    fn try_from(map: &'_ HeaderMap<HeaderValue>) -> Result<Self, HeaderParseError> {
        match parse_map(map) {
            Ok(v) => Ok(v),
            Err(why) => {
                // Now, there's a couple pairs of reasons we could have an error
                // here.
                //
                // The first set of reasons is:
                //
                // - Some headers are present, but not all;
                // - A required header is present, but it's just not very
                //   utf8y; or
                // - A required header is present, but it doesn't parse to the
                //   necessary type.
                //
                // In these cases, it's a legitimate error with the headers and
                // we should disregard it.
                //
                // The second set is:
                //
                // - The route isn't ratelimited (at least, not locally).
                //
                // This means that none of the headers are present. If that's
                // the case, then it's not limited (except for the global, of
                // course).

                let headers = &[
                    "x-ratelimit-bucket",
                    "x-ratelimit-limit",
                    "x-ratelimit-remaining",
                    "x-ratelimit-reset",
                ];

                if headers.iter().any(|k| map.contains_key(*k)) {
                    Err(why)
                } else if map.contains_key("x-ratelimit-global") {
                    Ok(Self::GlobalLimited {
                        reset_after: header_int(map, "x-ratelimit-reset-after")?,
                    })
                } else {
                    Ok(Self::None)
                }
            }
        }
    }
}

#[allow(clippy::cast_possible_truncation)]
fn parse_map(map: &HeaderMap<HeaderValue>) -> Result<Headers, HeaderParseError> {
    let bucket = header_str(map, "x-ratelimit-bucket")
        .ok()
        .map(ToOwned::to_owned);
    let global = header_bool(map, "x-ratelimit-global").unwrap_or(false);
    let limit = header_int(map, "x-ratelimit-limit")?;
    let remaining = header_int(map, "x-ratelimit-remaining")?;
    let reset = header_float(map, "x-ratelimit-reset")?;
    #[allow(clippy::cast_sign_loss)]
    let reset = (reset * 1000.).ceil() as u64;
    let reset_after = header_float(map, "x-ratelimit-reset-after")?;
    #[allow(clippy::cast_sign_loss)]
    let reset_after = (reset_after * 1000.).ceil() as u64;

    Ok(Headers::Present {
        bucket,
        global,
        limit,
        remaining,
        reset,
        reset_after,
    })
}

fn header_bool(map: &HeaderMap<HeaderValue>, name: &'static str) -> Result<bool, HeaderParseError> {
    let value = map
        .get(name)
        .ok_or(HeaderParseError::HeaderMissing { name })?;

    let text = value
        .to_str()
        .map_err(|source| HeaderParseError::HeaderNotUtf8 {
            name,
            source,
            value: value.as_bytes().to_owned(),
        })?;

    let end = text
        .parse()
        .map_err(|source| HeaderParseError::ParsingBoolText {
            name,
            source,
            text: text.to_owned(),
        })?;

    Ok(end)
}

fn header_float(map: &HeaderMap<HeaderValue>, name: &'static str) -> Result<f64, HeaderParseError> {
    let value = map
        .get(name)
        .ok_or(HeaderParseError::HeaderMissing { name })?;

    let text = value
        .to_str()
        .map_err(|source| HeaderParseError::HeaderNotUtf8 {
            name,
            source,
            value: value.as_bytes().to_owned(),
        })?;

    let end = text
        .parse()
        .map_err(|source| HeaderParseError::ParsingFloatText {
            name,
            source,
            text: text.to_owned(),
        })?;

    Ok(end)
}

fn header_int(map: &HeaderMap<HeaderValue>, name: &'static str) -> Result<u64, HeaderParseError> {
    let value = map
        .get(name)
        .ok_or(HeaderParseError::HeaderMissing { name })?;

    let text = value
        .to_str()
        .map_err(|source| HeaderParseError::HeaderNotUtf8 {
            name,
            source,
            value: value.as_bytes().to_owned(),
        })?;

    let end = text
        .parse()
        .map_err(|source| HeaderParseError::ParsingIntText {
            name,
            source,
            text: text.to_owned(),
        })?;

    Ok(end)
}

fn header_str<'a>(map: &'a HeaderMap<HeaderValue>, name: &'static str) -> Result<&'a str, HeaderParseError> {
    let value = map
        .get(name)
        .ok_or(HeaderParseError::HeaderMissing { name })?;

    let text = value
        .to_str()
        .map_err(|source| HeaderParseError::HeaderNotUtf8 {
            name,
            source,
            value: value.as_bytes().to_owned(),
        })?;

    Ok(text)
}
