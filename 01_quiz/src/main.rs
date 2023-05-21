use std::{error::Error, fs, io, path::PathBuf, process};

use clap::Parser;
use serde::Deserialize;

#[derive(Parser)]
struct Cli {
    #[arg(long)]
    path: PathBuf,
}

#[derive(Debug, Deserialize)]
struct QuizEntry {
    question: String,
    answer: String,
}

fn quiz(quiz_entries: &Vec<QuizEntry>) -> Result<(), Box<dyn Error>> {
    let mut correct_count = 0;
    let mut incorrect_count = 0;

    let mut guess = String::new();
    for quiz_entry in quiz_entries {
        println!("{}", quiz_entry.question);
        io::stdin().read_line(&mut guess).expect("Failed to read line from stdin");
        if guess.trim().eq(&quiz_entry.answer) {
            println!("Correct!");
            correct_count += 1;
        } else {
            println!("Wrong! The answer to {} is {}, not {}", quiz_entry.question, quiz_entry.answer, guess.trim());
            incorrect_count += 1
        }
        guess.clear();
    }

    let overall_count = correct_count + incorrect_count;
    println!("Result: {}/{} correct", correct_count, overall_count);

    Ok(())
}

fn parse_csv(content: String) -> Result<Vec<QuizEntry>, Box<dyn Error>> {
    let mut rdr = csv::Reader::from_reader(content.as_bytes());
    let mut results = vec![];
    for result in rdr.deserialize() {
        let record: QuizEntry = result?;
        results.push(record);
    }
    Ok(results)
}

fn main() {
    let args = Cli::parse();
    let content = fs::read_to_string(&args.path).expect("could not read file");
    let quiz_entries = parse_csv(content)
        .unwrap_or_else(|e| panic!("Failed to parse csv file: {:?} due to {}", args.path, e));

    if let Err(err) = quiz(&quiz_entries) {
        eprintln!("Error during quiz: {}", err);
        process::exit(1);
    }
}
