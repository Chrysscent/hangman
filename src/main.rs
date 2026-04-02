use rand::Rng;
use std::collections::HashSet;
use std::fs::File;
use std::io::{self, Read, Write};

fn main() {
    // load all words from file
    let words = load_words().expect("Failed to load words");

    // Select a random word
    let word = get_word(&words);

    // Start the Hangman game
    let mut hangman = GameState::new(word);
    hangman.run(&words);
}

struct GameState {
    secret: Vec<char>,
    hidden: Vec<char>,
    guess_counter: usize,
    warnings: usize,
    available_letters: HashSet<char>,
}

enum Input {
    Guess(char),
    Hint,
    Invalid(InputError),
}

enum InputError {
    Unavailable,
    NotALetter,
    NotSingleChar,
    Empty,
}

impl GameState {
    fn new(secret_word: &String) -> Self {
        let secret: Vec<char> = secret_word.chars().collect();
        let hidden: Vec<char> = vec!['_'; secret.len()];

        Self {
            secret,
            hidden,
            guess_counter: 6,
            warnings: 3,
            available_letters: ('a'..='z').collect(),
        }
    }

    fn run(&mut self, wordlist: &[String]) {
        println!("Welcome to the game Hangman!");
        println!(
            "I think of a word, it is {} letters long.",
            self.secret.len()
        );
        println!("Accumulating three warnings will lose you a guess.\n");

        while self.guess_counter > 0 {
            if self.is_won() {
                println!("\n\n🎉 Congrats! You guessed my word.");
                println!(
                    "Your total score: {}",
                    self.guess_counter * self.num_unique_letters()
                );
                return;
            }
            self.display_status();

            match self.read_input() {
                Input::Hint => {
                    show_possible_matches(&self.hidden, wordlist);
                }
                Input::Invalid(err) => self.handle_invalid(err),
                Input::Guess(ch) => self.process_guess(ch),
            }
        }

        println!("\n\n💀 Sorry you ran out of guesses.");
        println!("The word is {}", self.secret.iter().collect::<String>());
    }

    fn is_won(&self) -> bool {
        self.hidden == self.secret
    }

    fn num_unique_letters(&self) -> usize {
        self.secret.iter().collect::<HashSet<_>>().len()
    }

    fn display_status(&self) {
        let mut ordered_alphabets = self
            .available_letters
            .clone()
            .into_iter()
            .collect::<Vec<char>>();
        ordered_alphabets.sort();
        println!("\n\n- - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -");
        if self.guess_counter != 1 {
            println!("You have {} guesses left. Good luck!", self.guess_counter);
        } else {
            println!("You have {} guess left. Good luck!", self.guess_counter);
        }
        println!(
            "Available letters: {}",
            ordered_alphabets.iter().collect::<String>()
        );
    }

    fn read_input(&mut self) -> Input {
        print!("Guess a letter(* for hint): ");
        io::stdout().flush().expect("Failed to flush stdout");

        let mut input: String = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read user input");

        let trimmed_input = input.trim();
        if trimmed_input == "*" {
            return Input::Hint;
        }

        if trimmed_input.len() > 1 {
            return Input::Invalid(InputError::NotSingleChar);
        }

        if trimmed_input.is_empty() {
            return Input::Invalid(InputError::Empty);
        }

        let guess = trimmed_input.chars().next().unwrap();
        if !guess.is_ascii_alphabetic() {
            return Input::Invalid(InputError::NotALetter);
        }

        let guess = guess.to_ascii_lowercase();
        if !self.available_letters.remove(&guess) {
            return Input::Invalid(InputError::Unavailable);
        }

        Input::Guess(guess)
    }

    fn handle_invalid(&mut self, err: InputError) {
        if self.warnings == 0 {
            self.guess_counter -= 1;
            println!("{}", err.message());
            println!("You have no warnings left. Sorry, you will lose a guess.");
            return;
        }

        self.warnings -= 1;

        if self.warnings == 1 {
            println!(
                "{} You now have {} warning left",
                err.message(),
                self.warnings
            )
        } else {
            println!(
                "{} You now have {} warnings left",
                err.message(),
                self.warnings
            );
        }
        println!("word to guess: {}", self.hidden.iter().collect::<String>());
    }

    fn process_guess(&mut self, guess: char) {
        let mut good_guess = false;

        for (i, &v) in self.secret.iter().enumerate() {
            if v == guess {
                good_guess = true;
                self.hidden[i] = guess;
            }
        }

        if good_guess {
            println!("Good guess: {}", self.hidden.iter().collect::<String>());
        } else {
            println!(
                "Sorry, that letter is not in my word: {}",
                self.hidden.iter().collect::<String>()
            );
            if is_vowel(guess) {
                self.guess_counter -= 2;
                println!("You entered a vowel letter, that will cost you 2 guesses.");
            } else {
                self.guess_counter -= 1;
                println!("You entered a consonant letter, you will lose a guess.");
            }
        }
    }
}

impl InputError {
    fn message(&self) -> &'static str {
        match self {
            InputError::Unavailable => "⚠️ You have already guessed that letter.",
            InputError::NotALetter => "⚠️ Invalid input. Please enter letters only.",
            InputError::NotSingleChar => "⚠️ Please enter one letter at a time.",
            InputError::Empty => "⚠️ Please make a guess.",
        }
    }
}

fn show_possible_matches(hidden: &[char], wordlist: &[String]) {
    let pattern: String = hidden.iter().collect();
    print!("\nPossible word matches are: ");

    for word in wordlist {
        if match_with_pattern(&pattern, word) {
            print!("{} ", word);
        }
    }
    println!();
}

fn match_with_pattern(pattern: &str, word: &str) -> bool {
    pattern.len() == word.len()
        && pattern
            .chars()
            .zip(word.chars())
            .all(|(p, w)| p == '_' || p == w)
}

fn is_vowel(guess: char) -> bool {
    matches!(guess, 'a' | 'e' | 'i' | 'o' | 'u')
}

/// Loads words from "words.txt" and returns them as a vector
fn load_words() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut f: File = File::open("words.txt")?;
    let mut buffer: String = String::new();
    f.read_to_string(&mut buffer)?;

    println!("\nLoading word list from file...");

    // Split text by spaces, remove empty entries, and collect into a Vec<String>
    let word_list: Vec<String> = buffer
        .split(' ')
        .filter(|word: &&str| !word.is_empty())
        .map(|word: &str| word.to_string())
        .collect();

    println!("  {} words loaded.", word_list.len());
    Ok(word_list)
}

/// Returns a random word from the word list
fn get_word(list: &[String]) -> &String {
    let mut rng: rand::prelude::ThreadRng = rand::rng();
    let random_num: usize = rng.random_range(0..list.len());
    &list[random_num]
}
