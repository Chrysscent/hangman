use rand::Rng;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::io::{self, Write};

/// Stores the result of validating a user's input
struct InputInfo {
    valid: bool,
    msg: String,
}

fn main() {
    // load all words from file
    let words: Vec<String> = load_words().unwrap();

    // Select a random word
    let word: &String = get_word(&words);

    // Start the Hangman game
    hangman(word, &words);
}

/// Runs the main Hangman game loop
fn hangman(secret_word: &String, wordlist: &Vec<String>) {
    // Count unique letters in the secret word
    let mut unique_letters: HashMap<char, i32> = HashMap::with_capacity(secret_word.len());
    for char in secret_word.chars() {
        unique_letters
            .entry(char)
            .and_modify(|item| *item += 1)
            .or_insert(1);
    }

    // Buffer word used to track unrevealed letters
    let mut buf_word: String = secret_word.clone();

    // Create the hidden word using underscores
    let mut hidden: String = String::with_capacity(secret_word.len());
    for i in 0..secret_word.len() {
        hidden.insert(i, '_');
    }

    // Initial game resources
    let mut guess_counter: usize = 6;
    let mut good_guesses: usize = 0;
    let mut warnings: u32 = 3;
    let mut letters: String = "abcdefghijklmnopqrstuvwxyz".to_string();

    println!("Welcome to the game Hangman!");
    println!(
        "I think of a word, it is {} letters long.",
        secret_word.len()
    );
    println!("Accumulating three warnings will lose you a guess.\n");

    loop {
        // Lose condition
        if guess_counter == 0 {
            println!("Sorry you ran out of guesses. The word is {}", secret_word);
            break;
        }
        // Win condition
        if good_guesses == secret_word.len() {
            println!("Congrats! You guessed my word.");
            println!("Your total score: {}", guess_counter * unique_letters.len());
            break;
        }

        println!("- - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -");

        // Display remaining guesses with correct grammar
        if guess_counter != 1 {
            println!("You have {} guesses left. Good luck!", guess_counter);
        } else {
            println!("You have {} guess left. Good luck!", guess_counter);
        }

        println!("Available letters: {}", letters);
        print!("Guess a letter: ");
        io::stdout().flush().expect("Failed to flush stdout");

        // Handle user input
        let mut input: String = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read user input");

        // Ensure input is a single character
        if input.trim().len() > 1 {
            println!("\nPlease enter one letter at a time.");
            continue;
        };

        let guess: char = input.chars().next().unwrap().to_ascii_lowercase(); //  Convert input String to lowercase char
        let input_check: InputInfo = is_valid(guess, &mut letters);
        let mut text: String = String::from("warnings");

        // Handle invalid input and repeated guesses
        if !input_check.valid {
            if warnings == 0 {
                guess_counter -= 1;
                println!("\nYou have no warnings left. Sorry, you will lose a guess.");
                continue;
            }

            warnings -= 1;

            // Grammar handling
            if warnings == 1 {
                text = "warning".to_string();
            }

            if input_check.msg == "already guessed letter" {
                println!(
                    "\nYou have already guessed that letter. You now have {} {} left:",
                    warnings, text
                );
                println!("Word to guess: {}", hidden);
                continue;
            } else if input_check.msg == "not a letter" {
                println!(
                    "\nInvalid input. Please enter letters only, You now have {} {} left:",
                    warnings, text
                );
                println!("Word to guess: {}", hidden);
                continue;
            }
        }

        // Handle hint request
        if input_check.msg == "hint req" {
            show_possible_matches(&hidden, wordlist);
            continue;
        }

        // Handle incorrect guesses
        if !guess_in_word(guess, &secret_word) {
            println!("\nSorry, that letter is not in my word: {}", hidden);
            match guess {
                'a' | 'e' | 'i' | 'o' | 'u' => {
                    // Vowel penalty
                    guess_counter -= 2;
                    println!("You entered a vowel letter, that will cost you 2 guesses.");
                }
                _ => {
                    // Consonant penalty
                    guess_counter -= 1;
                    println!("You entered a consonant letter, you will lose a guess.")
                }
            };
            continue;
        }

        // Reveal correctly guessed letters
        reveal_guess(guess, &mut hidden, &mut buf_word, &mut good_guesses);
    }
}

/// Loads words from "words.txt" and returns them as a vector
fn load_words() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut f: File = File::open("words.txt")?;
    let mut buffer: String = String::new();
    f.read_to_string(&mut buffer)?;

    println!("Loading word list from file...");

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
fn get_word(list: &Vec<String>) -> &String {
    let mut rng: rand::prelude::ThreadRng = rand::rng();
    let random_num: usize = rng.random_range(0..list.len());
    &list[random_num]
}

/// Validates user input and updates available letters
fn is_valid(guess: char, alphabets: &mut String) -> InputInfo {
    let mut msg: InputInfo = InputInfo {
        valid: false,
        msg: String::with_capacity(30),
    };

    match guess {
        'a'..='z' => {
            match alphabets.find(guess) {
                Some(x) => {
                    msg.valid = true;
                    msg.msg = String::from("valid guess");
                    alphabets.remove(x);
                }
                None => {
                    msg.msg = String::from("already guessed letter");
                }
            };
        }
        '*' => {
            msg.valid = true;
            msg.msg = String::from("hint req");
        }
        _ => msg.msg = String::from("not a letter"),
    };

    msg
}

/// Checks if a word matches the current hidden pattern
fn match_with_hidden(my_word: &String, other_word: &String) -> bool {
    if my_word.len() != other_word.len() {
        return false;
    }

    let char_list: Vec<char> = other_word.chars().collect();
    let mut matched: bool = false;

    for (i, v) in my_word.chars().enumerate() {
        match v {
            'a'..='z' => {
                if v == char_list[i] {
                    matched = true;
                } else {
                    matched = false;
                    break;
                }
            }
            '_' => {}
            _ => {}
        }
    }

    matched
}

/// Displays all possible matching words based on the current hidden word
fn show_possible_matches(my_word: &String, wordlist: &Vec<String>) {
    print!("\nPossible word matches are: ");
    for word in wordlist {
        if match_with_hidden(my_word, word) {
            print!("{} ", word);
        }
    }
    println!();
}

/// Returns true if the guessed character exists in the secret word
fn guess_in_word(guess: char, secret_word: &String) -> bool {
    secret_word.find(guess).is_some()
}

/// Updates the hidden display string and removes found letters from the buffer word
fn reveal_guess(guess: char, hidden: &mut String, buffer: &mut String, good_guesses: &mut usize) {
    loop {
        match buffer.find(guess) {
            Some(x) => {
                hidden.remove(x);
                hidden.insert(x, guess);

                buffer.remove(x);
                buffer.insert(x, '_');

                *good_guesses += 1;
            }
            None => {
                break;
            }
        }
    }

    println!("\nGood guess: {}", hidden);
}
