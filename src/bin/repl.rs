// SPDX-License-Identifier: Apache-2.0

#![warn(missing_docs)]

//! A quick + dirty little REPL (Read + Eval + Print Loop) command line tool to
//! verify if single or multi line input is valid OGC CQL2 expression, or not.
//!
//! Entering the sequence of two tildas `~~` followed by `↵` (the \[ENTER\] key)
//! initiates a multi-line mode which ends when `Ctrl-D` is pressed. In this
//! mode consecutive input is concatenated into one string before processing.
//!
//! The program will first attempt to parse the input as a TEXT based expression
//! of CQL2. If it fails, it will try again treating it as JSON. In either
//! cases if it succeeds it will output an intermediary representation of the
//! expression. On the other hand, if it fails, an error message (in
//! <font color="red">red</font>) will be printed to `stderr`.
//!
//! To start the loop enter...
//! ```bash
//! cargo run --bin repl↵
//! ```
//! To exit the program, press `Ctrl-D`.
//!

use ogc_cql2::{Expression, MyError};
use std::io::{self, Write};

#[doc(hidden)]
const RED: &str = "\x1b[31m";
#[doc(hidden)]
const GREEN: &str = "\x1b[32m";
#[doc(hidden)]
const YELLOW: &str = "\x1b[33m";
#[doc(hidden)]
const RESET: &str = "\x1b[0m";
#[doc(hidden)]
const MULTILINE: &str = "~~";

macro_rules! error {
    ( $( $arg: tt )* ) => {
        {
            let msg = ::std::fmt::format(::core::format_args!($($arg)*));
            eprintln!("{RED}{msg}{RESET}");
        }
    }
}

macro_rules! info {
    ( $( $arg: tt )* ) => {
        {
            let msg = ::std::fmt::format(::core::format_args!($($arg)*));
            println!("{YELLOW}{msg}{RESET}");
        }
    }
}

macro_rules! note {
    ( $( $arg: tt )* ) => {
        {
            let msg = ::std::fmt::format(::core::format_args!($($arg)*));
            println!("{GREEN}{msg}{RESET}");
        }
    }
}

#[doc(hidden)]
fn prompt(s: &str) -> Result<(), MyError> {
    print!("{GREEN}{s} {RESET}");
    io::stdout().flush().map_err(MyError::IO)
}

/// Executable main method.
///
/// Invoke it like so...
/// ```bash
/// cargo run --bin repl↵
/// ```
fn main() -> Result<(), MyError> {
    note!("Enter a text or JSON CQL2 expression to verify.\nWhen done, hit Ctrl-D.");
    let stdin = io::stdin();
    loop {
        prompt("> ")?;
        let mut line = String::new();
        match stdin.read_line(&mut line) {
            Ok(0) => {
                note!("\nSee you later...");
                break;
            }

            Ok(_) => {
                let first = line.trim();
                let input = if first == MULTILINE {
                    info!("Enter multi-line mode. Exit w/ Ctrl-D");
                    let mut lines = String::new();
                    loop {
                        prompt(">>")?;
                        let mut next = String::new();
                        match stdin.read_line(&mut next) {
                            Ok(0) => break,
                            Ok(_) => lines.push_str(&next),
                            Err(x) => {
                                error!("Failed Read: {}", x);
                                break;
                            }
                        }
                    }
                    lines
                } else {
                    first.to_owned()
                };

                let expr = Expression::try_from_text(&input);
                match expr {
                    Ok(x) => note!("OK! {}", x),
                    Err(x) => {
                        error!("Failed as TEXT: {}.", x);
                        info!("Will try as JSON...");
                        let expr = Expression::try_from_json(&input);
                        match expr {
                            Ok(x) => note!("OK! {}", x),
                            Err(x) => error!("Failed as JSON: {}", x),
                        }
                    }
                }
            }
            Err(x) => {
                error!("Failed Read: {}", x);
                break;
            }
        }
    }

    Ok(())
}
