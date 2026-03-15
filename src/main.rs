#![allow(unused)]

use eyre::Result;
use std::{io::Write, path::Path};
use yansi::Paint;

use loxide::{
    ast::{Expr, ExprKind, Stmt},
    environment::Environment,
    error::{HandleLoxResult, HandleLoxResultIter},
    interpreter::{Interpreter, LoxValue},
    parser::Parser,
    scanner::Scanner,
    source::Source,
    token::Token,
};

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
                app.options.print_ast = true;
            }
            Long("print-nil-result") => {
                app.options.print_nil_result = true;
            }
            Long("plain") => {
                app.options.plain = true;
                yansi::disable();
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
            if !self.options.plain {
                println!(
                    "• {} running {}\n",
                    "loxide".yellow(),
                    file.blue().underline()
                );
            }

            let file = file.as_ref();
            let script = std::fs::read_to_string(file)?;
            let mut env = Environment::default();
            return run_script(&script, Some(file), &mut env, &self.options);
        } else {
            if !self.options.plain {
                println!(
                    "• {} in {} mode\n",
                    "loxide".yellow(),
                    "REPL".green().underline()
                );
            }

            let mut sources = elsa::FrozenVec::new();
            let mut env = Environment::default();
            loop {
                print!("> ");
                std::io::stdout().flush()?;

                let mut buffer = String::new();
                let n = std::io::stdin().read_line(&mut buffer)?;
                if n == 0 {
                    break;
                }
                sources.push(buffer);
                // trim ending newline if any
                let trimmed = sources.last().unwrap().trim_end_matches("\n");

                let _ = run_script(trimmed, None, &mut env, &self.options)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct RunnerOptions {
    print_tokens: bool,
    print_ast: bool,
    print_nil_result: bool,
    plain: bool,
}

impl Default for RunnerOptions {
    fn default() -> Self {
        Self {
            print_tokens: false,
            print_ast: false,
            print_nil_result: false,
            plain: false,
        }
    }
}

fn run_script<'env, 'src>(
    script: &'src str,
    location: Option<&'src Path>,
    env: &'env mut Environment<'src>,
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
        if options.plain {
            println!("Parsing errors: {}", total_errors);
        } else {
            println!(
                "\n{}  Parsing errors: {}",
                "🮮".bright_red(),
                total_errors.to_string().bright_white(),
            );
        }
        return Ok(());
    }

    match Interpreter::execute_many(ast, source, env).report_err() {
        Some(value @ LoxValue::Nil) if options.print_nil_result => {
            if options.plain {
                println!("{}", value);
            } else {
                println!("{} {}", "•".green().dim(), value.to_string().green());
            }
        }
        Some(LoxValue::Nil) => {}
        Some(value) => {
            if options.plain {
                println!("{}", value);
            } else {
                println!("{} {}", "•".green().dim(), value.to_string().green());
            }
        }
        _ => {
            if options.plain {
                println!("Runtime errors: {}", 1);
            } else {
                println!("\n{}  Runtime errors: {}", "🮮".dim(), 1.to_string());
            }
        }
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
    I: IntoIterator<Item = &'i Stmt<'src>>,
    'src: 'i,
{
    println!(
        "{} {:^5} {}",
        "─".repeat(3).cyan(),
        "AST".cyan(),
        "─".repeat(3).cyan()
    );
    for (i, stmt) in ast.into_iter().enumerate() {
        println!("{}: {}", format!("{i:02}").dim(), stmt.to_string().italic());
    }
}
