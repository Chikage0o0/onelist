use serde_json::Value;
use snafu::Snafu;

use super::Thumbnails;

pub fn parse_thumb(value: &Value) -> Result<Thumbnails, Error> {
    let thumb = if let Some(thumbnails) = value.get(0) {
        let small = thumbnails
            .get("small")
            .and_then(|v| v.get("url"))
            .and_then(|v| v.as_str());
        let medium = thumbnails
            .get("medium")
            .and_then(|v| v.get("url"))
            .and_then(|v| v.as_str());
        let large = thumbnails
            .get("large")
            .and_then(|v| v.get("url"))
            .and_then(|v| v.as_str());

        match (small, medium, large) {
            (Some(small), Some(medium), Some(large)) => Thumbnails {
                small: small.to_string(),
                medium: medium.to_string(),
                large: large.to_string(),
            },
            _ => {
                return Err(Error::ParseError {
                    value: value.to_string(),
                })
            }
        }
    } else {
        return Err(Error::ParseError {
            value: value.to_string(),
        });
    };

    Ok(thumb)
}

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
    #[snafu(display("No thumbnails found for location: {}", value))]
    ParseError { value: String },
}
