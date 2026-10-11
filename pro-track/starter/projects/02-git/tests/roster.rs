//! Checks the roster's rules. Don't change this file: the grader checks that it
//! is untouched.

#[test]
fn the_team_has_a_real_captain() {
    let captain = roster::captain();
    assert!(!captain.trim().is_empty(), "the captain can't be empty");
    assert_ne!(captain, "TBD", "pick a captain (task 3 in the README)");
}

#[test]
fn the_captain_is_on_the_team() {
    assert!(
        roster::PLAYERS.contains(&roster::captain()),
        "the captain must be one of PLAYERS, not {:?}",
        roster::captain()
    );
}

#[test]
fn a_match_allows_five_players() {
    assert_eq!(roster::MAX_PLAYERS, 5, "the rules allow exactly 5 players");
    assert!(roster::PLAYERS.len() <= roster::MAX_PLAYERS);
}

#[test]
fn the_summary_names_the_captain() {
    assert!(roster::summary().contains(roster::captain()));
}
