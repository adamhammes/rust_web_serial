use scraper::{Html, Selector};
use time::OffsetDateTime;
use url::Url;

use crate::{template::ChapterTemplate, BookInfo};

use super::{extract_text, parse_iso, EpubSource};

pub struct RoyalRoad {}

impl EpubSource for RoyalRoad {
    fn matches_url(&self, url: &Url) -> bool {
        match url.domain() {
            Some(domain) => domain == "www.royalroad.com",
            None => false,
        }
    }

    fn parse_index(&self, url: &Url, document: &Html) -> BookInfo {
        let author_selector = Selector::parse("[href^='/profile']").unwrap();
        let title_selector = Selector::parse("h1").unwrap();

        let author_name = extract_text(document.select(&author_selector).next().unwrap());
        let title = extract_text(document.select(&title_selector).next().unwrap());

        let chapter_selector = Selector::parse(".chapter-row > td:first-of-type a").unwrap();
        let chapters = document
            .select(&chapter_selector)
            .map(|elem| url.join(elem.value().attr("href").unwrap()).unwrap())
            .collect::<Vec<_>>();

        let chapter_time_selector = Selector::parse(".chapter-row time").unwrap();
        let modified = document
            .select(&chapter_time_selector)
            .map(|elem| elem.value().attr("datetime").unwrap())
            .map(|datetime| parse_iso(datetime))
            .max()
            .unwrap();

        let fic_id = url.path_segments().unwrap().nth(1).unwrap();
        let fic_slug = url.path_segments().unwrap().nth(2).unwrap();
        let book_id = format!("royalroad-{fic_id}");

        let cover_image_url =
            format!("https://www.royalroadcdn.com/public/covers-large/{fic_id}-{fic_slug}.jpg");
        let cover_image_url = Url::parse(&cover_image_url);

        BookInfo {
            id: book_id,
            author_name,
            title,
            chapters,
            cover: cover_image_url.ok(),
            modified,
        }
    }

    fn parse_chapter(&self, document: &Html) -> ChapterTemplate {
        let content_selector = Selector::parse(".chapter-content").unwrap();
        let title_selector = Selector::parse("h1").unwrap();
        let published_at_selector = Selector::parse("[title='Published'] ~ time").unwrap();

        let text = document
            .select(&content_selector)
            .next()
            .unwrap()
            .html()
            // TODO: yikes
            .replace("<br>", "<br />")
            .replace("<hr>", "<hr />");

        let title = extract_text(document.select(&title_selector).next().unwrap());
        let published_at_elem = document.select(&published_at_selector).next().unwrap();
        let published_at_unix = published_at_elem
            .value()
            .attr("unixtime")
            .unwrap()
            .parse::<i64>()
            .unwrap();

        ChapterTemplate {
            text,
            title,
            published_at: OffsetDateTime::from_unix_timestamp(published_at_unix).unwrap(),
        }
    }
}
