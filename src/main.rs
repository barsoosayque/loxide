#![allow(unused)]

use eyre::Result;
use std::{io::Write, path::Path};
use yansi::Paint;

use crate::{
    ast::{Expr, ExprKind},
    error::{HandleLoxResult, HandleLoxResultIter},
    interpreter::Interpreter,
    parser::Parser,
    scanner::Scanner,
    source::Source,
    token::Token,
};

mod ast;
mod error;
mod interpreter;
mod parser;
mod scanner;
mod source;
mod token;

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
            Long("print-ast") => {
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
            println!(
                "• {} running {}\n",
                "loxide".yellow(),
                file.blue().underline()
            );

            let file = file.as_ref();
            let script = std::fs::read_to_string(file)?;
            return run_script(&script, Some(file), &self.options);
        } else {
            println!(
                "• {} in {} mode\n",
                "loxide".yellow(),
                "REPL".green().underline()
            );

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
    print_ast: bool,
}

impl Default for RunnerOptions {
    fn default() -> Self {
        Self {
            print_tokens: false,
            print_ast: false,
        }
    }
}

fn run_script<'src>(
    script: &'src str,
    location: Option<&'src Path>,
    options: &RunnerOptions,
) -> Result<()> {
    let source = Source {
        script: script.as_ref(),
        location,
    };

    let (tokens, scanner_errors) = Scanner::scan(&source).process();
    if options.print_tokens {
        print_tokens(&tokens);
    }

    let (ast, parser_errors) = Parser::parse(tokens, &source).process();
    if options.print_ast {
        print_ast(&ast);
    }

    let total_errors = scanner_errors + parser_errors;
    if total_errors > 0 {
        println!(
            "\n{}  Parsing errors: {}",
            "🮮".bright_red(),
            total_errors.to_string().bright_white(),
        );
        return Ok(());
    }

    if let Some(value) = Interpreter::interpret(ast, &source).report_err() {
        println!("{} {}", "•".green().dim(), value.to_string().green());
    } else {
        println!("\n{}  Runtime errors: {}", "🮮".dim(), 1.to_string());
    }

    Ok(())
}

fn print_tokens<'src, 'i, I>(tokens: I)
where
    I: IntoIterator<Item = &'i Token<'src>>,
    'src: 'i,
{
    println!(
        "{} {:^5} {}",
        "─".repeat(3).magenta(),
        "Tokens".magenta(),
        "─".repeat(3).magenta()
    );
    for (i, token) in tokens.into_iter().enumerate() {
        println!(
            "{}: {}",
            format!("{i:02}").dim(),
            token.to_string().italic()
        );
    }
}

fn print_ast<'src, 'i, I>(ast: I)
where
    I: IntoIterator<Item = &'i Expr<'src>>,
    'src: 'i,
{
    println!(
        "{} {:^5} {}",
        "─".repeat(3).cyan(),
        "AST".cyan(),
        "─".repeat(3).cyan()
    );
    for (i, expr) in ast.into_iter().enumerate() {
        println!("{}: {}", format!("{i:02}").dim(), expr.to_string().italic());
    }
}
