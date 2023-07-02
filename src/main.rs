use std::{collections::HashMap, fs::File, io::Write};

use askama::Template;
use bytes::Bytes;
use futures::{stream, StreamExt};
use reqwest::Error;
use scraper::Html;
use time::OffsetDateTime;
use url::Url;
use zip::{result::ZipResult, write::FileOptions};

use crate::template::{BookContents, Navigation};

use crate::source::{EpubSource, RoyalRoad};

mod source;
mod template;

#[derive(Debug)]
pub struct BookInfo {
    id: String,
    title: String,
    author_name: String,
    chapters: Vec<Url>,
    cover: Option<Url>,
    modified: OffsetDateTime,
}

async fn bulk_download(requests: &[Url]) -> Vec<String> {
    let client = reqwest::Client::new();

    let responses = stream::iter(requests)
        .map(|req| {
            let client = &client;
            async move {
                let resp = client.get(req.to_string()).send().await?;
                resp.text().await
            }
        })
        .buffered(3)
        .collect::<Vec<_>>()
        .await;

    responses
        .into_iter()
        .filter_map(|result| result.ok())
        .collect::<Vec<_>>()
}

fn write_epub(nav: &Navigation, contents: &BookContents) -> ZipResult<()> {
    let file_options = FileOptions::default().compression_method(zip::CompressionMethod::Stored);

    let file_name = format!("output/{} - {}.epub", contents.author, contents.title);
    let file = File::create(file_name).unwrap();
    let mut zip = zip::ZipWriter::new(file);

    zip.start_file("mimetype", file_options)?;
    zip.write_all(include_bytes!("../templates/mimetype"))?;

    zip.add_directory("META-INF", file_options)?;
    zip.start_file("META-INF/container.xml", file_options)?;
    zip.write_all(include_bytes!("../templates/container.xml"))?;

    zip.add_directory("OEBPS", file_options)?;
    zip.start_file("OEBPS/nav.xhtml", file_options)?;
    zip.write_all(nav.render().unwrap().as_bytes())?;

    zip.start_file("OEBPS/content.opf", file_options)?;
    zip.write_all(contents.render().unwrap().as_bytes())?;

    zip.add_directory("OEBPS/content", file_options)?;
    for (i, body) in nav.chapters.iter().enumerate() {
        let chapter_index = i + 1;
        zip.start_file(
            format!("OEBPS/content/chapter_{chapter_index}.html"),
            file_options,
        )?;
        zip.write_all(body.render().unwrap().as_bytes())?;
    }

    zip.start_file("OEBPS/content/cover.html", file_options)?;
    zip.write_all(include_bytes!("../templates/cover.html"))?;

    zip.start_file("OEBPS/content/cover.jpg", file_options)?;

    if let Some(cover) = &contents.cover_image {
        zip.write_all(cover)?;
    }

    Ok(())
}

async fn download_url(url: &Url) -> Result<Html, Error> {
    let res = reqwest::get(url.to_string()).await?;
    let test = res.text().await?;
    Ok(Html::parse_document(&test))
}

async fn download_bytes(url: &Url) -> Result<Bytes, Error> {
    Ok(reqwest::get(url.to_string()).await?.bytes().await?)
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    let args: Vec<_> = std::env::args().collect();
    let url = Url::parse(&args[1]).unwrap();

    let sources: Vec<&dyn EpubSource> = vec![&RoyalRoad {}];

    let maybe_source = sources.iter().find(|&&s| s.matches_url(&url));
    let confirmed_source = match maybe_source {
        Some(source) => source,
        None => {
            println!("Failed to find source for url {url}");
            return Ok(());
        }
    };

    let document = download_url(&url).await?;
    let book_meta = confirmed_source.parse_index(&url, &document);

    let cover = match book_meta.cover {
        Some(url) => Some(download_bytes(&url).await?),
        None => None,
    };

    let chapters = bulk_download(&book_meta.chapters[..2]).await;

    let bodies = chapters
        .into_iter()
        .map(|html| confirmed_source.parse_chapter(&Html::parse_document(&html)))
        .collect::<Vec<_>>();

    let tox = Navigation {
        id: &book_meta.id,
        title: &book_meta.title,
        chapters: &bodies,
    };

    let content = BookContents {
        id: book_meta.id.as_ref(),
        title: &book_meta.title,
        language: "en-US".to_owned(),
        author: &book_meta.author_name,
        modified: book_meta.modified,
        cover_image: cover,
        meta: HashMap::from([("author".to_owned(), book_meta.author_name.to_owned())]),
        chapters: &bodies,
    };

    write_epub(&tox, &content).unwrap();

    Ok(())
}
