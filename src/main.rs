use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::fmt::Debug;
use std::result::Result;
// https://github.com/charlesreid1/five-letter-words/blob/master/sgb-words.txt

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
    while !w.solved() {
        println!("wordl = {:?}", w);
        let g = w.guess().expect("while generating next guess");
        println!("guess={}", g);
        println!("which indexes were exactly correct: [0, 1, 2, 3, 4] ");
        let mut buffer = String::new();
        io::stdin()
            .read_line(&mut buffer)
            .expect("while reading line");
        w.markCorrectIndices(&buffer);
        println!("which letters were present");
        let mut buffer = String::new();
        io::stdin()
            .read_line(&mut buffer)
            .expect("while reading line");
        w.markNegatives(&buffer);
    }
}

#[derive(Debug)]
enum WordlError {
    GuessesExhausted,
    IOError(io::Error),
}

struct Wordl {
    word: [Option<char>; 5],
    negatives: [BTreeSet<char>; 5],
    dictionary: BTreeSet<String>,
    last: Option<String>,
}

impl Debug for Wordl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Wordl")
            .field("word", &self.word)
            .field("negatives", &self.negatives)
            .field("last", &self.last)
            .finish()
    }
}

impl Wordl {
    fn markCorrectIndices(&mut self, i: &str) {
        let i = i.trim();
        let last = self.last.as_ref().unwrap();
        i.chars()
            .map(|c| c.to_digit(10).unwrap() as usize)
            .for_each(|i| {
                self.word[i] = Some(last.chars().nth(i).unwrap());
            });
    }

    fn markNegatives(&mut self, i: &str) {
        let i = i.trim();
        i.chars().for_each(|i| {
            self.negatives.insert(i);
        });
    }

    fn solved(&self) -> bool {
        5 == self.word.iter().filter(|w| w.is_some()).count()
    }

    fn guess(&mut self) -> Result<String, WordlError> {
        let a = {
            let score = |a: &String| {
                let o = self.overlaps(a);
                let u = self.unique(a);
                // depending on how many rounds remain, alter the strategy
                // two strategies to consider: (a) find the most unique guesses, and (b) overlap with the most success
                // let's do (a) for now.
                // TODO add dictionary frequency counter
                u * 5 + o
            };
            let a = self
                .dictionary
                .iter()
                .filter(|a| {
                    for (idx, w) in self.word.iter().enumerate() {
                        if let Some(c) = w {
                            let b = a
                                .chars()
                                .nth(idx)
                                .expect("dictionary word was not as long as expected");
                            if b != *c {
                                return false;
                            }
                        }
                    }
                    true
                })
                .max_by(|a, b| {
                    let sa = score(a);
                    let sb = score(b);
                    if sa == sb {
                        Ordering::Equal
                    } else if sa < sb {
                        Ordering::Less
                    } else {
                        Ordering::Greater
                    }
                });
            a
        };
        match a {
            Some(s) => {
                self.last = Some(s.clone());
                // self.dictionary.remove(s); // TODO figure out how to satisfy the borrow checker here
                Ok(s.clone())
            }
            None => Err(WordlError::GuessesExhausted),
        }
    }

    // overlaps returns the number of characters in the provided string which match the known
    // letters.
    fn overlaps(&self, a: &str) -> u32 {
        self.word.iter().zip(a.chars()).fold(0, |acc, (owc, ac)| {
            if let Some(wc) = owc {
                if *wc == ac {
                    return acc + 1;
                }
            }
            acc
        })
    }

    // unique returns the number of unique characters in the provided string a which have not
    // been previously guessed.
    fn unique(&self, a: &str) -> u32 {
        let invalid: BTreeSet<char> = self
            .word
            .iter()
            .filter_map(|w| {
                if let Some(c) = w {
                    return Some(*c);
                }
                None
            })
            .collect();
        let mut seen: BTreeSet<char> = BTreeSet::default();
        a.chars().fold(0, |acc, c| {
            if invalid.contains(&c) || self.negatives.contains(&c) || seen.contains(&c) {
                return acc;
            }
            seen.insert(c);
            acc + 1
        })
    }
}

impl Default for Wordl {
    fn default() -> Self {
        Wordl {
            word: [None; 5],
            negatives: [BTreeSet::default(); 5],
            dictionary: BTreeSet::default(),
            last: None,
        }
    }
}

// guess: strep
// greens?
// yellows?
// guess: {}
