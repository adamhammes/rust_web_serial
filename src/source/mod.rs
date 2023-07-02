use scraper::{ElementRef, Html};
use time::{format_description::well_known::Iso8601, OffsetDateTime};
use url::Url;

use crate::{template::ChapterTemplate, BookInfo};

pub trait EpubSource {
    fn matches_url(&self, book_url: &Url) -> bool;
    fn parse_index(&self, url: &Url, document: &Html) -> BookInfo;
    fn parse_chapter(&self, document: &Html) -> ChapterTemplate;
}

fn extract_text(elem: ElementRef) -> String {
    elem.text().collect::<String>().trim().into()
}

fn parse_iso(input: &str) -> OffsetDateTime {
    OffsetDateTime::parse(input, &Iso8601::PARSING).unwrap()
}

mod royal_road;
pub use crate::source::royal_road::RoyalRoad;
