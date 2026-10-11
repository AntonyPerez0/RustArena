//! What the four functions must return, on small and medium inputs.
//! Provided: don't change this file (the grader checks that). Every optimization you make
//! must keep these passing: run `cargo test` after each change.

use std::collections::{BTreeMap, HashSet};

use hotspots::{Row, data, ranks, to_csv, top_words, unique_in_order};

fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

fn counts(pairs: &[(&str, usize)]) -> Vec<(String, usize)> {
    pairs.iter().map(|&(w, n)| (w.to_string(), n)).collect()
}

// ----- unique_in_order -----

#[test]
fn unique_keeps_first_occurrences_in_order() {
    let items = strings(&["b", "a", "b", "c", "a"]);
    assert_eq!(unique_in_order(&items), strings(&["b", "a", "c"]));
    assert_eq!(unique_in_order(&[]), Vec::<String>::new());
    assert_eq!(unique_in_order(&strings(&["x", "x", "x"])), strings(&["x"]));
    assert_eq!(
        unique_in_order(&strings(&["", "", "A", "a"])),
        strings(&["", "A", "a"])
    );
}

#[test]
fn unique_matches_a_reference_on_generated_names() {
    let names = data::names(3_000, 2_000, 11);
    let mut seen = HashSet::new();
    let expected: Vec<String> = names.iter().filter(|n| seen.insert(*n)).cloned().collect();
    assert_eq!(unique_in_order(&names), expected);
}

// ----- top_words -----

#[test]
fn top_words_counts_case_insensitively() {
    let text = "the cat and THE dog. The end, and... that's it";
    assert_eq!(
        top_words(text, 3),
        counts(&[("the", 3), ("and", 2), ("cat", 1)]),
        "ties (cat, dog, end, it, s, that) are alphabetical"
    );
}

#[test]
fn top_words_edge_cases() {
    assert_eq!(top_words("", 5), counts(&[]));
    assert_eq!(top_words(" ,.;- \n", 5), counts(&[]));
    assert_eq!(top_words("one two two", 0), counts(&[]));
    assert_eq!(top_words("b a b", 10), counts(&[("b", 2), ("a", 1)]));
    assert_eq!(
        top_words("Café café CAFÉ 42 42x", 3),
        counts(&[("café", 3), ("42", 1), ("42x", 1)])
    );
}

#[test]
fn top_words_matches_a_reference_on_generated_text() {
    let text = data::text(4_000, 300, 12);
    let mut by_word: BTreeMap<String, usize> = BTreeMap::new();
    for word in text
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
    {
        *by_word.entry(word.to_lowercase()).or_default() += 1;
    }
    let mut expected: Vec<(String, usize)> = by_word.into_iter().collect();
    expected.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    expected.truncate(25);
    assert_eq!(top_words(&text, 25), expected);
}

// ----- to_csv -----

#[test]
fn csv_has_a_header_and_one_line_per_row() {
    let rows = [
        Row {
            name: "ada".to_string(),
            score: 90,
        },
        Row {
            name: "linus".to_string(),
            score: 7,
        },
    ];
    assert_eq!(to_csv(&rows), "name,score\nada,90\nlinus,7\n");
    assert_eq!(to_csv(&[]), "name,score\n");
}

#[test]
fn csv_matches_a_reference_on_generated_rows() {
    let rows = data::rows(2_000, 13);
    let mut expected = String::from("name,score\n");
    for row in &rows {
        expected += &format!("{},{}\n", row.name, row.score);
    }
    assert_eq!(to_csv(&rows), expected);
}

// ----- ranks -----

#[test]
fn ranks_are_competition_ranks() {
    assert_eq!(ranks(&[50, 80, 80, 20]), [3, 1, 1, 4]);
    assert_eq!(ranks(&[]), Vec::<usize>::new());
    assert_eq!(ranks(&[7]), [1]);
    assert_eq!(ranks(&[5, 5, 5]), [1, 1, 1]);
    assert_eq!(ranks(&[1, 2, 3, 4]), [4, 3, 2, 1]);
    assert_eq!(ranks(&[0, u32::MAX, 0]), [2, 1, 2]);
}

#[test]
fn ranks_match_a_reference_on_generated_scores() {
    let scores = data::scores(1_500, 400, 14);
    let expected: Vec<usize> = scores
        .iter()
        .map(|&s| 1 + scores.iter().filter(|&&other| other > s).count())
        .collect();
    assert_eq!(ranks(&scores), expected);
}
