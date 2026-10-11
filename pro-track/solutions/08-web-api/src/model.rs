//! The JSON the API reads and writes. serde's derives generate the conversion
//! code; the field names are the JSON keys.

use serde::{Deserialize, Serialize};

use crate::AppError;

/// A stored bookmark, as the API returns it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: u64,
    pub url: String,
    pub title: String,
    pub tags: Vec<String>,
}

/// The body of `POST /bookmarks`. `tags` may be left out.
#[derive(Debug, Clone, Deserialize)]
pub struct NewBookmark {
    pub url: String,
    pub title: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// The body of `PATCH /bookmarks/{id}`: only the fields that are present change.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct BookmarkPatch {
    pub url: Option<String>,
    pub title: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// The query string of `GET /bookmarks`, like `?tag=rust&q=async&limit=10&offset=20`.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ListParams {
    pub tag: Option<String>,
    pub q: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// The response of `GET /bookmarks`: one page of items, and how many match in total.
#[derive(Debug, Clone, Serialize)]
pub struct Page {
    pub items: Vec<Bookmark>,
    pub total: usize,
}

const MAX_URL: usize = 2000;
const MAX_TITLE: usize = 200;
const MAX_TAGS: usize = 10;
const MAX_TAG: usize = 30;

impl NewBookmark {
    /// Checks the rules and returns the bookmark normalized: url and title
    /// trimmed, tags sorted without duplicates.
    pub fn validate(self) -> Result<NewBookmark, AppError> {
        let invalid = |message: &str| Err(AppError::Validation(message.to_string()));

        let url = self.url.trim().to_string();
        let has_scheme = ["http://", "https://"].iter().any(|scheme| {
            url.strip_prefix(scheme)
                .is_some_and(|rest| !rest.is_empty())
        });
        if !has_scheme || url.chars().any(char::is_whitespace) {
            return invalid("url must be an http:// or https:// URL");
        }
        if url.chars().count() > MAX_URL {
            return invalid("url must be at most 2000 characters");
        }

        let title = self.title.trim().to_string();
        if title.is_empty() {
            return invalid("title must not be empty");
        }
        if title.chars().count() > MAX_TITLE {
            return invalid("title must be at most 200 characters");
        }

        let mut tags = self.tags;
        tags.sort();
        tags.dedup();
        if tags.len() > MAX_TAGS {
            return invalid("at most 10 tags");
        }
        if let Some(bad) = tags.iter().find(|tag| !valid_tag(tag)) {
            return invalid(&format!("invalid tag \"{bad}\""));
        }

        Ok(NewBookmark { url, title, tags })
    }
}

fn valid_tag(tag: &str) -> bool {
    (1..=MAX_TAG).contains(&tag.len())
        && tag
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
