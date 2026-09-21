use clap::{Parser as ClapParser, Subcommand};
use colored::Colorize;
use sankode_borrowck::BorrowChecker;
use sankode_eval::Interpreter;
use sankode_lexer::Lexer;
use sankode_parser::Parser;
use sankode_semantics::TypeChecker;
use std::fs;
use std::path::PathBuf;

#[derive(ClapParser)]
#[command(name = "sankode")]
#[command(author = "tovganesh")]
#[command(version = "0.1.0")]
#[command(about = "सङ्कोड (Sankode) - Pure Devanagari Systems Programming Language", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Directly run a script if provided without subcommand
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Execute a Sankode source file (.सङ् / .सङ्स्कृ)
    Run {
        #[arg(value_name = "FILE")]
        path: PathBuf,
    },
    /// Verify types and borrow safety without executing
    Check {
        #[arg(value_name = "FILE")]
        path: PathBuf,
    },
    /// Print tokens produced by the Devanagari lexer
    Tokens {
        #[arg(value_name = "FILE")]
        path: PathBuf,
    },
    /// Parse and display the Abstract Syntax Tree (AST)
    Parse {
        #[arg(value_name = "FILE")]
        path: PathBuf,
    },
    /// Launch the interactive REPL (सङ्वादकम्)
    Repl,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Run { path }) => run_file(&path),
        Some(Commands::Check { path }) => check_file(&path),
        Some(Commands::Tokens { path }) => print_tokens(&path),
        Some(Commands::Parse { path }) => print_ast(&path),
        Some(Commands::Repl) => run_repl(),
        None => {
            if let Some(file) = cli.file {
                run_file(&file);
            } else {
                run_repl();
            }
        }
    }
}

fn run_file(path: &PathBuf) {
    let program = load_and_verify(path);
    let mut interpreter = Interpreter::new();
    interpreter.load_program(&program);
    if let Err(e) = interpreter.run_main() {
        eprintln!("{} {}", "निष्पादने दोषः (Runtime error):".red().bold(), e);
        std::process::exit(1);
    }
}

fn check_file(path: &PathBuf) {
    let _ = load_and_verify(path);
    println!(
        "{} {}",
        "✓".green().bold(),
        "निर्दोषः सङ्केतः (Type and borrow safety checks passed!)".green()
    );
}

fn load_and_verify(path: &PathBuf) -> sankode_core::Program {
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{} संचिका न प्राप्ता (File error): {}", "दोषः:".red().bold(), e);
            std::process::exit(1);
        }
    };

    let mut lexer = Lexer::new(&source);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{} {}", "पदच्छेदे दोषः (Lexer error):".red().bold(), e);
            std::process::exit(1);
        }
    };

    let mut parser = Parser::new(tokens);
    let program = match parser.parse_program() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{} {}", "वाक्यरचनादोषः (Parser error):".red().bold(), e);
            std::process::exit(1);
        }
    };

    let mut type_checker = TypeChecker::new();
    if let Err(e) = type_checker.check_program(&program) {
        eprintln!("{} {}", "प्रकारदोषः (Type error):".red().bold(), e);
        std::process::exit(1);
    }

    let mut borrow_checker = BorrowChecker::new();
    if let Err(e) = borrow_checker.check_program(&program) {
        eprintln!("{} {}", "स्वामित्वदोषः (Borrow error):".red().bold(), e);
        std::process::exit(1);
    }

    program
}

fn print_tokens(path: &PathBuf) {
    let source = fs::read_to_string(path).expect("Failed to read file");
    let mut lexer = Lexer::new(&source);
    match lexer.tokenize() {
        Ok(tokens) => {
            println!("{}", "=== सङ्केताः (Tokens) ===".cyan().bold());
            for tok in tokens {
                println!("{:15} {:?} at {}", tok.kind.to_string().yellow(), tok.kind, tok.span);
            }
        }
        Err(e) => eprintln!("{} {}", "दोषः:".red().bold(), e),
    }
}

fn print_ast(path: &PathBuf) {
    let source = fs::read_to_string(path).expect("Failed to read file");
    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize().expect("Lexing failed");
    let mut parser = Parser::new(tokens);
    match parser.parse_program() {
        Ok(prog) => {
            println!("{}", "=== वाक्यवृक्षः (AST) ===".green().bold());
            println!("{:#?}", prog);
        }
        Err(e) => eprintln!("{} {}", "दोषः:".red().bold(), e),
    }
}

fn run_repl() {
    println!("{}", "=========================================================".cyan());
    println!("{}", "  ॥ सङ्कोड सङ्वादकम् ॥ (Sankode REPL v०.१.०)".yellow().bold());
    println!("{}", "  Sanskrit / Pure Devanagari Interactive Shell");
    println!("{}", "  समाप्त्यर्थं 'विराम' अथवा Ctrl+C लिखन्तु।");
    println!("{}", "=========================================================".cyan());

    let mut rl = rustyline::DefaultEditor::new().expect("Failed to initialize readline");
    let mut interpreter = Interpreter::new();

    loop {
        let readline = rl.readline("॥ सङ्कोड ॥ > ");
        match readline {
            Ok(line) => {
                let trimmed = line.trim();
                if trimmed == "विराम" || trimmed == "exit" || trimmed == "quit" {
                    println!("{}", "सङ्वादकः समाप्तः। (Goodbye!)".cyan());
                    break;
                }
                if trimmed.is_empty() {
                    continue;
                }
                let _ = rl.add_history_entry(line.as_str());

                // Wrap statement in main if needed or evaluate line
                let wrapped = if !trimmed.starts_with("क्रिया") {
                    format!("क्रिया मुख्य() -> रिक्त\n{}\nइति", trimmed)
                } else {
                    trimmed.to_string()
                };

                let mut lexer = Lexer::new(&wrapped);
                match lexer.tokenize() {
                    Ok(tokens) => {
                        let mut parser = Parser::new(tokens);
                        match parser.parse_program() {
                            Ok(prog) => {
                                interpreter.load_program(&prog);
                                if let Err(e) = interpreter.run_main() {
                                    eprintln!("{} {}", "दोषः:".red(), e);
                                }
                            }
                            Err(e) => eprintln!("{} {}", "वाक्यरचनादोषः:".red(), e),
                        }
                    }
                    Err(e) => eprintln!("{} {}", "पदच्छेदे दोषः:".red(), e),
                }
            }
            Err(_) => {
                println!("\n{}", "सङ्वादकः समाप्तः।".cyan());
                break;
            }
        }
    }
}
