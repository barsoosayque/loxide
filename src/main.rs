use eyre::Result;
use std::{io::Write, path::Path};

mod error;
mod scanner;

fn main() -> Result<()> {
    let mut args = std::env::args();
    let name = args.next().unwrap();

    match args.len() {
        0 => repl(),
        1 => run_file(args.next().unwrap()),
        _ => {
            println!("Usage: {name} [script]");
            std::process::exit(64);
        }
    }
}

fn repl() -> Result<()> {
    println!("// Repl mode //");

    let mut buffer = String::new();
    loop {
        print!("> ");
        std::io::stdout().flush().unwrap();

        let n = std::io::stdin().read_line(&mut buffer)?;
        if n == 0 {
            break;
        }
        let _ = run_script(&buffer, None)?;
        buffer.clear();
    }

    Ok(())
}

fn run_file(file: impl AsRef<Path>) -> Result<()> {
    let file = file.as_ref();
    println!("// Running {file:?} //");

    let script = std::fs::read_to_string(file)?;
    run_script(&script, Some(file))
}

fn run_script<'src>(script: &'src str, location: Option<&'src Path>) -> Result<()> {
    let source = script.as_ref();
    let scanner = scanner::Scanner::new(source, location);
    let tokens = scanner.scan()?;

    for token in tokens {
        println!("{token}");
    }

    Ok(())
}
