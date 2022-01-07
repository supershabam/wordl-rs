use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::fmt::Debug;

use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

// https://www.powerlanguage.co.uk/wordle/
// https://github.com/charlesreid1/five-letter-words/blob/master/sgb-words.txt
fn main() {
    let mut w = Wordl::default();
    if let Ok(lines) = read_lines("./words.txt") {
        // Consumes the iterator, returns an (Optional) String
        for line in lines {
            if let Ok(word) = line {
                w.dictionary.insert(word);
            }
        }
    }
    let words = vec![
        [
            Letter::Miss('a'),
            Letter::Miss('b'),
            Letter::Miss('a'),
            Letter::Contains('c'),
            Letter::Contains('i'),
        ],
        [
            Letter::Contains('c'),
            Letter::Miss('h'),
            Letter::Contains('i'),
            Letter::Miss('e'),
            Letter::Miss('f'),
        ],
        [
            Letter::Miss('d'),
            Letter::Hit('i'),
            Letter::Hit('c'),
            Letter::Miss('k'),
            Letter::Miss('s'),
        ],
        [
            Letter::Miss('l'),
            Letter::Hit('i'),
            Letter::Hit('c'),
            Letter::Miss('i'),
            Letter::Miss('t'),
        ],
    ];
    for word in words {
        for s in w.suggest(3) {
            println!("suggestion: {}", s);
        }
        println!("guessing {:?}", word);
        w.guess(word);
    }
    for s in w.suggest(3) {
        println!("suggestion: {}", s);
    }
}

#[derive(Debug)]
enum Letter {
    Hit(char),
    Miss(char),
    Contains(char),
}

type Word = [Letter; 5];

struct Wordl {
    dictionary: BTreeSet<String>,
    guesses: Vec<Word>,
}

impl Debug for Wordl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Wordl")
            .field("guesses", &self.guesses)
            .finish()
    }
}

impl Wordl {
    fn suggest(&self, upto: usize) -> Vec<String> {
        let mut v: Vec<String> = self.dictionary.iter().cloned().collect();
        v.sort_by(|a, b| Ordering::Equal);
        v.into_iter().take(upto).collect()
    }

    fn guess(&mut self, word: Word) {
        self.guesses.push(word);
        let guesses = &self.guesses;
        let valid = Wordl::make_is_valid(guesses);
        self.dictionary.retain(|k| valid(k));
    }

    fn make_contains(words: &Vec<Word>) -> Vec<char> {
        let instances: Vec<Vec<char>> = words
            .iter()
            .map(|w| {
                w.iter()
                    .filter_map(|l| match l {
                        &Letter::Contains(c) => Some(c),
                        _ => None,
                    })
                    .collect()
            })
            .collect();
        let result: Vec<char> = instances.iter().fold(vec![], |mut acc, instance| {
            let mut stack = acc.to_vec();
            for c in instance {
                if let Some(pos) = stack.iter().position(|s| *s == *c) {
                    stack.remove(pos);
                } else {
                    acc.push(*c);
                }
            }
            acc
        });
        result
    }

    fn make_hits(words: &Vec<Word>) -> [Option<char>; 5] {
        let mut result = [None; 5];
        for instance in words {
            for (idx, l) in instance.iter().enumerate() {
                if let Letter::Hit(c) = l {
                    result[idx] = Some(*c);
                }
            }
        }
        result
    }

    fn make_excludes_at(words: &Vec<Word>) -> [BTreeSet<char>; 5] {
        let mut result: [BTreeSet<char>; 5] = Default::default();
        for instance in words {
            for (idx, l) in instance.iter().enumerate() {
                if let Letter::Contains(c) = l {
                    result[idx].insert(*c);
                }
            }
        }
        result
    }

    fn make_is_valid(words: &Vec<Word>) -> Box<dyn Fn(&str) -> bool> {
        // expect these characters to be present somewhere in the string exactly once
        let contains = Wordl::make_contains(words);
        // hits are where known expected values are
        let hits = Wordl::make_hits(words);
        // expect none of these characters to be present in the string
        let excludes: BTreeSet<char> = words
            .iter()
            .flat_map(|word| word.iter())
            .filter_map(|l| match l {
                Letter::Miss(c) => Some(*c),
                _ => None,
            })
            .collect();
        let excludes_at = Wordl::make_excludes_at(words);

        Box::new(move |s: &str| -> bool {
            let mut contains = contains.to_vec();
            for (idx, c) in s.chars().enumerate() {
                // must execute first to muate the contains vector for each char in s
                if let Some(pos) = contains.iter().position(|cc| c == *cc) {
                    contains.remove(pos);
                }
                // the subsequent predicates may be re-ordered for efficiency
                if excludes.contains(&c) {
                    return false;
                }
                if let Some(h) = hits[idx] {
                    if h != c {
                        return false;
                    }
                }
                if excludes_at[idx].contains(&c) {
                    return false;
                }
            }
            if contains.len() > 0 {
                return false;
            }

            true
        })
    }
}

impl Default for Wordl {
    fn default() -> Self {
        Wordl {
            guesses: Vec::default(),
            dictionary: BTreeSet::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Letter;
    use crate::Wordl;

    #[test]
    fn test_valid() {
        let words = vec![
            [
                Letter::Miss('c'),
                Letter::Miss('c'),
                Letter::Contains('e'),
                Letter::Miss('c'),
                Letter::Miss('c'),
            ],
            [
                Letter::Contains('e'),
                Letter::Miss('c'),
                Letter::Contains('g'),
                Letter::Miss('c'),
                Letter::Miss('c'),
            ],
            [
                Letter::Contains('e'),
                Letter::Contains('g'),
                Letter::Contains('g'),
                Letter::Miss('c'),
                Letter::Miss('c'),
            ],
            [
                Letter::Miss('c'),
                Letter::Miss('c'),
                Letter::Miss('e'),
                Letter::Miss('c'),
                Letter::Contains('y'),
            ],
        ];
        let f = Wordl::make_is_valid(&words);
        assert_eq!(f(&"match"), false);
        assert_eq!(f(&"eggyy"), true);
    }

    #[test]
    fn contains_creates_expected_vector() {
        // _ _ E _ _
        // E _ G _ _
        // E G G _ _
        // Y _ _ _ _
        // -> EGGY

        let words = vec![
            [
                Letter::Miss('c'),
                Letter::Miss('c'),
                Letter::Contains('e'),
                Letter::Miss('c'),
                Letter::Miss('c'),
            ],
            [
                Letter::Contains('e'),
                Letter::Miss('c'),
                Letter::Contains('g'),
                Letter::Miss('c'),
                Letter::Miss('c'),
            ],
            [
                Letter::Contains('e'),
                Letter::Contains('g'),
                Letter::Contains('g'),
                Letter::Miss('c'),
                Letter::Miss('c'),
            ],
            [
                Letter::Miss('c'),
                Letter::Miss('c'),
                Letter::Miss('e'),
                Letter::Miss('c'),
                Letter::Contains('y'),
            ],
        ];
        assert_eq!(Wordl::make_contains(&words), vec!['e', 'g', 'g', 'y']);
    }
}
