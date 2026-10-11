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

impl Store {
    /// TODO (task 3): validate, refuse a url that is already stored
    /// (Conflict: "a bookmark with this url already exists"), store it with the
    /// next id and return it.
    pub fn insert(&mut self, new: NewBookmark) -> Result<Bookmark, AppError> {
        todo!("insert {new:?} after bookmark {} (task 3)", self.last_id)
    }

    /// TODO (task 3): the bookmark, or NotFound("bookmark 7 not found").
    pub fn get(&self, id: u64) -> Result<Bookmark, AppError> {
        todo!("get {id} from {} bookmarks (task 3)", self.bookmarks.len())
    }

    /// TODO (task 5): apply the fields present in `patch`, validate the result
    /// like a new bookmark, refuse a url used by *another* bookmark, and save.
    /// NotFound if there is no such bookmark.
    pub fn update(&mut self, id: u64, patch: BookmarkPatch) -> Result<Bookmark, AppError> {
        todo!("update {id} with {patch:?} (task 5)")
    }

    /// TODO (task 5): remove the bookmark, or NotFound.
    pub fn delete(&mut self, id: u64) -> Result<(), AppError> {
        todo!("delete {id} (task 5)")
    }

    /// TODO (task 6): the bookmarks that have tag `tag` (if given) and whose title
    /// contains `q` ignoring case (if given), in id order; `total` counts them
    /// all, `items` holds at most `limit` (default 20) after skipping `offset`
    /// (default 0). A limit outside 1..=100 is
    /// BadRequest("limit must be between 1 and 100").
    pub fn list(&self, params: &ListParams) -> Result<Page, AppError> {
        todo!("list with {params:?} (task 6)")
    }
}
