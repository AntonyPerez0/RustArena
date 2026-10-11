//! The data, in memory. Plain synchronous code: the handlers lock the store,
//! call one method and unlock, so no lock is ever held across an `.await`.

use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

use crate::AppError;
use crate::model::{Bookmark, BookmarkPatch, ListParams, NewBookmark, Page};

/// What the handlers share: many readers at once, or one writer.
pub type SharedStore = Arc<RwLock<Store>>;

#[derive(Debug, Default)]
pub struct Store {
    /// The last id handed out. Ids start at 1 and are never reused.
    last_id: u64,
    /// A BTreeMap keeps the bookmarks sorted by id.
    bookmarks: BTreeMap<u64, Bookmark>,
}

const DEFAULT_LIMIT: usize = 20;
const MAX_LIMIT: usize = 100;

fn not_found(id: u64) -> AppError {
    AppError::NotFound(format!("bookmark {id} not found"))
}

impl Store {
    /// Validates and stores a new bookmark with the next id.
    pub fn insert(&mut self, new: NewBookmark) -> Result<Bookmark, AppError> {
        let new = new.validate()?;
        self.check_url_is_free(&new.url, None)?;
        self.last_id += 1;
        let bookmark = Bookmark {
            id: self.last_id,
            url: new.url,
            title: new.title,
            tags: new.tags,
        };
        self.bookmarks.insert(bookmark.id, bookmark.clone());
        Ok(bookmark)
    }

    /// The bookmark with this id.
    pub fn get(&self, id: u64) -> Result<Bookmark, AppError> {
        self.bookmarks
            .get(&id)
            .cloned()
            .ok_or_else(|| not_found(id))
    }

    /// Applies the fields present in `patch`, validating the result like a new bookmark.
    pub fn update(&mut self, id: u64, patch: BookmarkPatch) -> Result<Bookmark, AppError> {
        let current = self.get(id)?;
        let merged = NewBookmark {
            url: patch.url.unwrap_or(current.url),
            title: patch.title.unwrap_or(current.title),
            tags: patch.tags.unwrap_or(current.tags),
        }
        .validate()?;
        self.check_url_is_free(&merged.url, Some(id))?;
        let updated = Bookmark {
            id,
            url: merged.url,
            title: merged.title,
            tags: merged.tags,
        };
        self.bookmarks.insert(id, updated.clone());
        Ok(updated)
    }

    /// Removes the bookmark with this id.
    pub fn delete(&mut self, id: u64) -> Result<(), AppError> {
        self.bookmarks
            .remove(&id)
            .map(|_| ())
            .ok_or_else(|| not_found(id))
    }

    /// One page of the bookmarks that match the filters, in id order.
    pub fn list(&self, params: &ListParams) -> Result<Page, AppError> {
        let limit = params.limit.unwrap_or(DEFAULT_LIMIT);
        if !(1..=MAX_LIMIT).contains(&limit) {
            return Err(AppError::BadRequest(
                "limit must be between 1 and 100".to_string(),
            ));
        }
        let query = params.q.as_deref().map(str::to_lowercase);
        let matching: Vec<&Bookmark> = self
            .bookmarks
            .values()
            .filter(|b| params.tag.as_ref().is_none_or(|tag| b.tags.contains(tag)))
            .filter(|b| {
                query
                    .as_ref()
                    .is_none_or(|q| b.title.to_lowercase().contains(q))
            })
            .collect();
        let total = matching.len();
        let items = matching
            .into_iter()
            .skip(params.offset.unwrap_or(0))
            .take(limit)
            .cloned()
            .collect();
        Ok(Page { items, total })
    }

    /// Conflict if another bookmark (not `except`) already has this url.
    fn check_url_is_free(&self, url: &str, except: Option<u64>) -> Result<(), AppError> {
        if self
            .bookmarks
            .values()
            .any(|b| b.url == url && Some(b.id) != except)
        {
            return Err(AppError::Conflict(
                "a bookmark with this url already exists".to_string(),
            ));
        }
        Ok(())
    }
}
