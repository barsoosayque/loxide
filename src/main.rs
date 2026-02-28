use eyre::Result;
use std::{io::Write, path::Path};
use yansi::Paint;

use crate::{error::LoxResultIter, scanner::Token};

mod error;
mod scanner;

fn main() -> Result<()> {
    use lexopt::prelude::*;

    let mut parser = lexopt::Parser::from_env();
    let bin_name = parser
        .bin_name()
        .unwrap_or(env!("CARGO_PKG_NAME"))
        .to_string();
    let version = env!("CARGO_PKG_VERSION");

    let mut app = App::default();
    while let Some(arg) = parser.next()? {
        match arg {
            Long("print-tokens") => {
                app.options.print_tokens = true;
            }
            Value(f) if app.file.is_none() => {
                app.file = Some(f.string()?);
            }
            Short('v') | Long("version") => {
                println!("{bin_name} v{version}");
            }
            _ => {
                println!("Usage: {bin_name} OPTIONS [file]");
                println!("");
                println!("OPTIONS:");
                println!("    --print-tokens:    Output scanned tokens to stdout");
                std::process::exit(64);
            }
        }
    }
    app.run()
}

#[derive(Debug, Default)]
struct App {
    options: RunnerOptions,
    file: Option<String>,
}

impl App {
    pub fn run(self) -> Result<()> {
        if let Some(file) = &self.file {
            println!("• {} {}:", "Loxide".yellow(), file.blue().underline());

            let file = file.as_ref();
            let script = std::fs::read_to_string(file)?;
            return run_script(&script, Some(file), &self.options);
        } else {
            println!("• {} {}:", "Loxide".yellow(), "REPL".green().underline());

            let mut buffer = String::new();
            loop {
                print!("> ");
                std::io::stdout().flush()?;

                let n = std::io::stdin().read_line(&mut buffer)?;
                if n == 0 {
                    break;
                }
                // trim ending newline if any
                let trimmed = buffer.trim_end_matches("\n");
                let _ = run_script(trimmed, None, &self.options)?;
                String::clear(&mut buffer);
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct RunnerOptions {
    print_tokens: bool,
}

impl Default for RunnerOptions {
    fn default() -> Self {
        Self { print_tokens: true }
    }
}

fn run_script<'src>(
    script: &'src str,
    location: Option<&'src Path>,
    options: &RunnerOptions,
) -> Result<()> {
    let source = script.as_ref();
    let scanner = scanner::Scanner::new(source, location);

    // consume iterator to process all of the errors before moving forward
    let tokens = scanner.scan().handle_errors(source).collect::<Vec<_>>();

    if options.print_tokens {
        print_tokens(tokens);
    }

    Ok(())
}

fn print_tokens<'src>(tokens: impl IntoIterator<Item = Token<'src>>) {
    for (i, token) in tokens.into_iter().enumerate() {
        println!(
            "{}: {}",
            format!("{i:02}").dim(),
            token.to_string().italic()
        );
    }
}
