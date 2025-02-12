use std::ops::Range;

use rand::Rng;

pub mod inty;
pub mod mixed;
pub mod stringy;

const COLLECTION_SIZE: Range<usize> = Range { start: 5, end: 15 };

/// Generate a random string with `n` amount of words, separated by a space.
pub fn words(rng: &mut impl Rng, n: usize) -> String {
    use rand::seq::IndexedRandom;
    names::NOUNS
        .choose_multiple(rng, n)
        .copied()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Generate a random word
pub fn word(rng: &mut impl Rng) -> String {
    use rand::seq::IndexedRandom;
    names::NOUNS.choose(rng).unwrap().to_string()
}
