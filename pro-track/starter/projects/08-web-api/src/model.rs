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

impl NewBookmark {
    /// TODO (task 3): check the rules, in this order, and return the bookmark
    /// normalized (url and title trimmed, tags sorted without duplicates).
    /// Each broken rule is an `AppError::Validation` with exactly this message:
    /// - url, trimmed: starts with `http://` or `https://`, has something after
    ///   that, and contains no whitespace: "url must be an http:// or https:// URL"
    /// - url: at most 2000 characters: "url must be at most 2000 characters"
    /// - title, trimmed: not empty: "title must not be empty"
    /// - title: at most 200 characters: "title must be at most 200 characters"
    /// - after removing duplicates, at most 10 tags: "at most 10 tags"
    /// - each tag: 1 to 30 lowercase ASCII letters, digits or `-`:
    ///   `invalid tag "Rust"` (with the tag in double quotes)
    ///
    /// Count characters with `.chars().count()`, not `.len()` (bytes).
    pub fn validate(self) -> Result<NewBookmark, AppError> {
        todo!("validate {self:?} (task 3)")
    }
}
