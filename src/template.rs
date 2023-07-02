use std::collections::HashMap;

use askama::Template;
use bytes::Bytes;
use time::OffsetDateTime;

mod filters {
    use time::{
        format_description::well_known::{
            iso8601::{Config, TimePrecision},
            Iso8601,
        },
        OffsetDateTime,
    };

    pub fn format_utc(date: &OffsetDateTime) -> ::askama::Result<String> {
        const PRECISION: TimePrecision = TimePrecision::Second {
            decimal_digits: None,
        };
        const FORMAT_CONFIG: u128 = Config::DEFAULT.set_time_precision(PRECISION).encode();
        let format = Iso8601::<{ FORMAT_CONFIG }>;
        Ok(date.format(&format).unwrap())
    }
}

#[derive(Debug, Template)]
#[template(path = "nav.xhtml", escape = "xml")]
pub struct Navigation<'a> {
    pub id: &'a str,
    pub title: &'a str,
    pub chapters: &'a [ChapterTemplate],
}

#[derive(Debug, Template)]
#[template(path = "content.opf", escape = "xml")]
pub struct BookContents<'a> {
    pub id: &'a str,
    pub title: &'a str,
    pub language: String,
    pub author: &'a str,
    pub modified: OffsetDateTime,
    pub cover_image: Option<Bytes>,
    pub meta: HashMap<String, String>,
    pub chapters: &'a [ChapterTemplate],
}

#[derive(Debug, Template)]
#[template(path = "chapter.html", escape = "xml")]
pub struct ChapterTemplate {
    pub text: String,
    pub title: String,
    pub published_at: OffsetDateTime,
}
