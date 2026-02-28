use eyre::Result;
use std::{io::Write, path::Path};
use yansi::Paint;

use crate::scanner::Token;

mod error;
mod scanner;

fn main() -> Result<()> {
    let mut args = std::env::args();
    let name = args.next().unwrap();

    let version = env!("CARGO_PKG_VERSION");
    match args.len() {
        0 => {
            println!(
                "• {}@{version} {}:",
                "Loxide".yellow(),
                "REPL".green().underline()
            );
            repl()
        }
        1 => {
            let file = args.next().unwrap();
            println!(
                "• {}@{version} {}:",
                "Loxide".yellow(),
                file.blue().underline()
            );
            run_file(file)
        }
        _ => {
            println!("Usage: {name} [script]");
            std::process::exit(64);
        }
    }
}

fn repl() -> Result<()> {
    let mut buffer = String::new();
    loop {
        print!("> ");
        std::io::stdout().flush()?;

        let n = std::io::stdin().read_line(&mut buffer)?;
        if n == 0 {
            break;
        }
        let _ = run_script(&buffer, None)?;
        String::clear(&mut buffer);
    }

    Ok(())
}

fn run_file(file: impl AsRef<Path>) -> Result<()> {
    let file = file.as_ref();
    let script = std::fs::read_to_string(file)?;
    run_script(&script, Some(file))
}

fn run_script<'src>(script: &'src str, location: Option<&'src Path>) -> Result<()> {
    let source = script.as_ref();
    let scanner = scanner::Scanner::new(source, location);
    let tokens = scanner.scan()?;

    print_tokens(tokens);

    Ok(())
}

#[allow(unused)]
fn print_tokens(tokens: Vec<Token>) {
    for (i, token) in tokens.iter().enumerate() {
        println!(
            "{}: {}",
            format!("{i:02}").dim(),
            token.to_string().italic()
        );
    }
}
