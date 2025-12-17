use rand::Rng;
use std::fs::File;
use std::io::Read;
use std::io::{self, Write};

// Struct to handle validation results of user input
struct ErrorMsg {
    valid: bool,
    msg: String,
}

fn main() {
    hangman();
}

/// Reads a space-separated list of words from "words.txt" and returns them as a Vector
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

/// Picks a random word from the provided word list
fn get_word(list: &Vec<String>) -> &String {
    let mut rng: rand::prelude::ThreadRng = rand::rng();
    let random_num: usize = rng.random_range(0..list.len());
    &list[random_num]
}

/// Checks if the input character is a lowercase letter and hasn't been used yet
fn is_valid(guess: char, alphabets: &mut String) -> ErrorMsg {
    let mut msg: ErrorMsg = ErrorMsg {
        valid: false,
        msg: String::with_capacity(30),
    };
    match guess {
        'a'..'z' => {
            match alphabets.find(guess) {
                Some(x) => {
                    // Character is valid and available; remove it from available letters
                    msg.valid = true;
                    msg.msg = String::from("no error");
                    alphabets.remove(x);
                }
                None => {
                    msg.valid = false;
                    msg.msg = String::from("already guessed letter");
                }
            };
        }
        _ => msg.msg = String::from("not a letter"),
    };
    msg
}

/// Returns true if the guessed character exists within the secret word
fn guess_in_word(guess: char, secret_word: &String) -> bool {
    match secret_word.find(guess) {
        Some(_x) => true,
        None => false,
    }
}

/// Updates the hidden display string and removes found letters from the buffer word
fn reveal_guess(guess: char, hidden: &mut String, buffer: &mut String, good_guesses: &mut usize) {
    loop {
        match buffer.find(guess) {
            Some(x) => {
                // Replace underscore with the letter in the display string
                hidden.remove(x);
                hidden.insert(x, guess);
                // Mark as found in the buffer so it isn't found again in the next loop iteration
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

/// Main game logic loop
fn hangman() {
    let words: Vec<String> = load_words().unwrap();
    let word: &String = get_word(&words);
    let mut buf_word: String = word.clone(); // Copy used to track remaining letters to find
    let mut good_guesses: usize = 0;

    // Initialize the hidden string with underscores
    let mut hidden: String = String::with_capacity(word.len());
    for i in 0..word.len() {
        hidden.insert(i, '_');
    }

    let mut guess_counter: usize = 6;
    let mut warnings: u32 = 3;
    let mut letters: String = "abcdefghijklmnopqrstuvwxyz".to_string();

    println!("Welcome to the game Hangman!");
    println!(
        "I think of a word, it is {} and {} letters long.",
        word,
        word.len()
    );
    println!("Accumulating three warnings will lose you a guess.\n");

    loop {
        // Lose condition
        if guess_counter == 0 {
            println!("Sorry you ran out of guesses. The word is {}", word);
            break;
        }
        // Win condition
        if good_guesses == word.len() {
            println!("Congrats! You guessed my word.");
            println!("Your total score: {}", guess_counter * word.len());
            break;
        }

        println!("- - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -");
        println!("You have {} guesses left. Good luck!", guess_counter);
        println!("Available letters: {}", letters);
        print!("Guess a letter: ");
        io::stdout().flush().expect("Failed to flush stdout");

        // Handle user input
        let mut input: String = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read user input");

        // Ensure input is not multiple characters
        if input.trim().len() > 1 {
            println!("\nPlease enter one letter at a time.");
            continue;
        };

        let guess: char = input.chars().next().unwrap().to_ascii_lowercase();
        let input_check: ErrorMsg = is_valid(guess, &mut letters);

        // Handle invalid inputs or repeats
        if !input_check.valid {
            if warnings == 0 {
                guess_counter -= 1;
                println!("\nYou have no warnings left. Sorry, you will lose a guess.");
                continue;
            }
            if input_check.msg == "already guessed letter" {
                warnings -= 1;
                println!(
                    "\nYou have already guessed that letter. You now have {} warnings left:",
                    warnings
                );
                println!("Word to guess: {}", hidden);
                continue;
            } else if input_check.msg == "not a letter" {
                warnings -= 1;
                println!(
                    "\nInvalid input. Please enter letters only, You now have {} warnings left:",
                    warnings
                );
                println!("Word to guess: {}", hidden);
                continue;
            }
        }

        // Check if the valid guess is actually in the secret word
        if !guess_in_word(guess, &word) {
            println!("\nSorry, that letter is not in my word: {}", hidden);
            match guess {
                'a' | 'e' | 'i' | 'o' | 'u' => {
                    // Vowels are more expensive
                    guess_counter -= 2;
                    println!("You entered a vowel letter, that will cost you 2 guesses.");
                }
                _ => {
                    guess_counter -= 1;
                    println!("You entered a consonant letter, you will lose a guess.")
                }
            };
            continue;
        }

        // If the guess is correct, reveal it in the hidden word
        reveal_guess(guess, &mut hidden, &mut buf_word, &mut good_guesses);
    }
}
