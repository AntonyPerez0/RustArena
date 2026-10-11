//! The team roster for the next tournament.
//!
//! Two teammates are about to change this file at the same time: see task 3 in
//! the README.

/// Everyone on the team, in the order they joined.
pub const PLAYERS: [&str; 5] = ["ferris", "nova", "kestrel", "atlas", "pixel"];

/// The most players a team may field in a match.
pub const MAX_PLAYERS: usize = 5;

/// Who leads the team. Must be one of [`PLAYERS`].
pub fn captain() -> &'static str {
    "TBD"
}

/// One line for the team page.
pub fn summary() -> String {
    format!("{} players, captain: {}", PLAYERS.len(), captain())
}
