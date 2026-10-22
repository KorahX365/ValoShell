//! VALORANT's news for the play screen's tiles: the articles playvalorant.com lists (trailers,
//! esports, patch notes), read from the data the news page embeds. The client's own feed is not
//! public, but it carries the same articles. Kept for half an hour per language; any failure
//! leaves the tiles on their store art.

use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, OnceLock, PoisonError};
use std::time::{Duration, Instant};

use serde::Serialize;
use serde_json::Value;
use ts_rs::TS;

const SITE: &str = "https://playvalorant.com";
const TIMEOUT: Duration = Duration::from_secs(10);
const FRESH_FOR: Duration = Duration::from_secs(30 * 60);
const ITEMS_KEPT: usize = 12;
/// The tiles are 240 px wide; the CDN scales on request, so a 4K header comes down at 40 KB.
const IMAGE_WIDTH: u32 = 640;
const DATA_TAG: &str = "<script id=\"__NEXT_DATA__\"";

/// One article.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[ts(export)]
pub struct NewsItem {
    pub title: String,
    pub image: String,
    /// Where it opens: the article, video or partner page.
    pub url: String,
    /// `esports`, `game_updates`, `announcements`...
    pub category: String,
    /// The site's one-line summary, as plain text; may be empty.
    pub summary: String,
    /// ISO 8601.
    pub published_at: String,
}

/// The newest articles, newest first, and the current patch.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, TS)]
#[ts(export)]
pub struct NewsView {
    pub items: Vec<NewsItem>,
    /// "13.08", from the newest patch notes.
    pub patch_version: Option<String>,
    pub patch_url: Option<String>,
}

/// The news in `locale` ("es-es"; anything malformed reads as "en-us"), from the cache when
/// fresh. `None` when the site cannot be read.
pub async fn news(locale: &str) -> Option<NewsView> {
    let locale = clean_locale(locale);
    if let Some(view) = cached(&locale) {
        return Some(view);
    }
    let view = parse_news(&fetch_page(&locale).await?)?;
    cache()
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .insert(locale, (Instant::now(), view.clone()));
    Some(view)
}

fn cache() -> &'static Mutex<HashMap<String, (Instant, NewsView)>> {
    static CACHE: OnceLock<Mutex<HashMap<String, (Instant, NewsView)>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn cached(locale: &str) -> Option<NewsView> {
    let cache = cache().lock().unwrap_or_else(PoisonError::into_inner);
    cache
        .get(locale)
        .filter(|(at, _)| at.elapsed() < FRESH_FOR)
        .map(|(_, view)| view.clone())
}

/// "es-ES" -> "es-es"; only two-letter pairs reach the URL.
fn clean_locale(raw: &str) -> String {
    let locale = raw.trim().to_ascii_lowercase().replace('_', "-");
    let is_pair = locale.len() == 5
        && locale.as_bytes()[2] == b'-'
        && locale
            .bytes()
            .enumerate()
            .all(|(at, byte)| at == 2 || byte.is_ascii_lowercase());
    if is_pair { locale } else { "en-us".to_owned() }
}

async fn fetch_page(locale: &str) -> Option<String> {
    riot::tls::install_crypto_provider();
    let client = reqwest::Client::builder()
        .user_agent(concat!("ValoShell/", env!("CARGO_PKG_VERSION")))
        .timeout(TIMEOUT)
        .gzip(true)
        .build()
        .ok()?;
    let response = client
        .get(format!("{SITE}/{locale}/news/"))
        .send()
        .await
        .ok()?;
    if !response.status().is_success() {
        tracing::warn!(status = %response.status(), "news_fetch_failed");
        return None;
    }
    response.text().await.ok()
}

/// The articles in a news page's embedded data.
pub fn parse_news(html: &str) -> Option<NewsView> {
    let start = html.find(DATA_TAG)?;
    let body_start = start + html[start..].find('>')? + 1;
    let body_end = body_start + html[body_start..].find("</script>")?;
    let data: Value = serde_json::from_str(&html[body_start..body_end]).ok()?;
    let mut items = Vec::new();
    let mut seen = HashSet::new();
    collect(&data, &mut items, &mut seen);
    items.sort_by(|a, b| b.published_at.cmp(&a.published_at));
    let patch = items.iter().find_map(|item| {
        (item.category == "game_updates")
            .then(|| version_in(&item.title).map(|version| (version, item.url.clone())))
            .flatten()
    });
    items.retain(|item| version_in(&item.title).is_none() || item.category != "game_updates");
    items.truncate(ITEMS_KEPT);
    let (patch_version, patch_url) = patch.unzip();
    Some(NewsView {
        items,
        patch_version,
        patch_url,
    })
}

/// Every article card in the page data, once each (the page repeats them across sections).
fn collect(value: &Value, items: &mut Vec<NewsItem>, seen: &mut HashSet<String>) {
    match value {
        Value::Object(map) => {
            if let Some(item) = article(value)
                && seen.insert(item.title.clone())
            {
                items.push(item);
            }
            for child in map.values() {
                collect(child, items, seen);
            }
        }
        Value::Array(list) => {
            for child in list {
                collect(child, items, seen);
            }
        }
        _ => {}
    }
}

fn article(value: &Value) -> Option<NewsItem> {
    let title = value.get("title")?.as_str()?.trim();
    let published_at = value.get("publishedAt")?.as_str()?;
    let image = ["media", "imageMedia"]
        .iter()
        .find_map(|key| value.get(key)?.get("url")?.as_str())?;
    let link = value
        .get("action")
        .and_then(|action| action.get("payload")?.get("url")?.as_str())
        .unwrap_or_default();
    if title.is_empty() || !image.starts_with("https://") {
        return None;
    }
    Some(NewsItem {
        title: title.to_owned(),
        image: sized(image),
        url: absolute(link),
        category: value
            .get("category")
            .and_then(|category| category.get("machineName")?.as_str())
            .unwrap_or_default()
            .to_owned(),
        summary: value
            .get("description")
            .and_then(|description| description.get("body")?.as_str())
            .map(plain_text)
            .unwrap_or_default(),
        published_at: published_at.to_owned(),
    })
}

/// Summaries come as small HTML fragments: keep the words, drop the tags and entities.
fn plain_text(html: &str) -> String {
    let mut text = String::with_capacity(html.len());
    let mut is_tag = false;
    for c in html.chars() {
        match c {
            '<' => is_tag = true,
            '>' => {
                is_tag = false;
                text.push(' ');
            }
            _ if !is_tag => text.push(c),
            _ => {}
        }
    }
    let text = text
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'");
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn sized(image: &str) -> String {
    let joiner = if image.contains('?') { '&' } else { '?' };
    format!("{image}{joiner}w={IMAGE_WIDTH}")
}

/// Site-relative links ("/es-es/news/...") made whole; anything else kept as given.
fn absolute(link: &str) -> String {
    if link.starts_with('/') {
        format!("{SITE}{link}")
    } else {
        link.to_owned()
    }
}

/// The "13.08" in "Versión 13.08 - VALORANT" / "VALORANT Patch Notes 13.08".
fn version_in(title: &str) -> Option<String> {
    title
        .split(|c: char| !(c.is_ascii_digit() || c == '.'))
        .find_map(|word| {
            let (major, minor) = word.split_once('.')?;
            let is_version = (1..=2).contains(&major.len())
                && minor.len() == 2
                && major
                    .bytes()
                    .chain(minor.bytes())
                    .all(|byte| byte.is_ascii_digit());
            is_version.then(|| word.to_owned())
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn card(title: &str, category: &str, date: &str, link: &str) -> Value {
        serde_json::json!({
            "title": title,
            "publishedAt": date,
            "media": { "url": "https://cmsassets.rgpub.io/a.jpg?accountingTag=VAL" },
            "category": { "machineName": category },
            "description": { "type": "html", "body": "<p>New&nbsp;agent <b>out</b> now</p>" },
            "action": { "type": "weblink", "payload": { "url": link } },
        })
    }

    fn page(cards: &[Value]) -> String {
        let data = serde_json::json!({ "props": { "pageProps": { "blades": [{ "items": cards }, { "items": cards }] } } });
        format!(
            "<html><script id=\"__NEXT_DATA__\" type=\"application/json\">{data}</script></html>"
        )
    }

    #[test]
    fn articles_come_newest_first_once_each_with_the_patch_split_out() {
        let html = page(&[
            card(
                "Versión 13.08 - VALORANT",
                "game_updates",
                "2026-10-13T00:00:00Z",
                "/es-es/news/game-updates/valorant-patch-notes-13-08",
            ),
            card(
                "VOCÊ É BRABA // Game Changers Championship",
                "esports",
                "2026-10-20T00:00:00Z",
                "https://youtube.com/watch?v=1",
            ),
            card(
                "Versión 13.06 - VALORANT",
                "game_updates",
                "2026-09-22T00:00:00Z",
                "/x",
            ),
            card(
                "Hush trailer",
                "game_updates",
                "2026-10-18T00:00:00Z",
                "https://youtube.com/watch?v=2",
            ),
        ]);
        let view = parse_news(&html).unwrap();

        let titles: Vec<&str> = view.items.iter().map(|item| item.title.as_str()).collect();
        assert_eq!(
            titles,
            ["VOCÊ É BRABA // Game Changers Championship", "Hush trailer"]
        );
        assert_eq!(view.patch_version.as_deref(), Some("13.08"));
        assert_eq!(
            view.patch_url.as_deref(),
            Some("https://playvalorant.com/es-es/news/game-updates/valorant-patch-notes-13-08")
        );
        assert!(view.items[0].image.ends_with("accountingTag=VAL&w=640"));
        assert_eq!(view.items[0].summary, "New agent out now");
    }

    #[test]
    fn a_page_without_its_data_reads_as_nothing() {
        assert_eq!(parse_news("<html></html>"), None);
    }

    #[test]
    fn locales_are_cleaned_before_they_reach_the_url() {
        assert_eq!(clean_locale("es-ES"), "es-es");
        assert_eq!(clean_locale("pt_BR"), "pt-br");
        assert_eq!(clean_locale("../../etc"), "en-us");
        assert_eq!(clean_locale(""), "en-us");
    }

    #[test]
    fn versions_need_two_minor_digits() {
        assert_eq!(
            version_in("Versión 13.08 - VALORANT").as_deref(),
            Some("13.08")
        );
        assert_eq!(
            version_in("VALORANT Patch Notes 9.11").as_deref(),
            Some("9.11")
        );
        assert_eq!(version_in("VCT 2027: la gira mundial"), None);
    }
}
