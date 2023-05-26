use std::{error::Error, fs, io, path::PathBuf, process};
use std::thread;
use std::sync::mpsc::{Sender, Receiver, TryRecvError};
use std::sync::mpsc;
use std::time::Instant;

use clap::Parser;
use serde::Deserialize;

#[derive(Parser)]
struct Cli {
    #[arg(short = 'p', long = "path")]
    path: PathBuf,

    #[arg(short = 's', long = "seconds", default_value_t = 30)]
    seconds: u64,
}

#[derive(Debug, Deserialize)]
struct QuizEntry {
    question: String,
    answer: String,
}

fn quiz(quiz_entries: &Vec<QuizEntry>, time_limit_in_seconds: u64) -> Result<(), Box<dyn Error>> {
    let instant = Instant::now();
    let mut correct_count = 0;
    let overall_count = quiz_entries.len();

    'outer: for quiz_entry in quiz_entries {
        let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();
        let question = quiz_entry.question.clone();
        thread::spawn(move || {
            let mut guess = String::new();
            println!("{}", question);
            io::stdin().read_line(&mut guess).expect("Failed to read line from stdin");
            tx.send(guess.clone()).unwrap();
            guess.clear();
        });
        loop {
            if instant.elapsed().as_secs() > time_limit_in_seconds {
                println!("Time is up");
                break 'outer;
            }
            match rx.try_recv() {
                Err(TryRecvError::Empty) => {},
                Err(_error) => {
                    panic!("Error when trying to receive answer")
                },
                Ok(answer) =>{
                    if answer.trim().eq(&quiz_entry.answer) {
                        println!("Correct!");
                        correct_count += 1;
                    } else {
                        println!("Wrong! The answer to {} is {}, not {}", quiz_entry.question, quiz_entry.answer, answer.trim());
                    }
                    break;
                }
            }
        }
    }

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

    if let Err(err) = quiz(&quiz_entries, args.seconds) {
        eprintln!("Error during quiz: {}", err);
        process::exit(1);
    }
}
