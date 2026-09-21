use clap::Parser as ClapParser;
use colored::Colorize;
use sankode_eval::{Interpreter, Value};
use sankode_ime::Transliterator;
use sankode_lexer::Lexer;
use sankode_parser::Parser;
use std::fs;
use std::path::PathBuf;

#[derive(ClapParser)]
#[command(name = "sanskipt")]
#[command(author = "tovganesh")]
#[command(version = "0.1.0")]
#[command(about = "सङ्स्कृ (Sanskipt) - Python-inspired Dynamic Scripting Runtime for Sankode", long_about = None)]
struct Cli {
    /// Script file to execute (.सङ्स्कृ / .सङ्)
    #[arg(value_name = "SCRIPT")]
    file: Option<PathBuf>,

    /// Program passed in as string (terminates option list, like python -c)
    #[arg(short = 'c', long = "command", value_name = "CODE")]
    command: Option<String>,

    /// Enable Roman phonetic input mode (transliterates ITRANS/Roman to Devanagari)
    #[arg(long = "ime")]
    ime: bool,
}

fn main() {
    let cli = Cli::parse();

    if let Some(code) = cli.command {
        let actual_code = if cli.ime {
            Transliterator::to_devanagari(&code)
        } else {
            code
        };
        run_code(&actual_code);
    } else if let Some(file) = cli.file {
        run_file(&file, cli.ime);
    } else {
        run_repl(cli.ime);
    }
}

fn run_file(path: &PathBuf, ime: bool) {
    let raw = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{} संचिका न प्राप्ता (File error): {}", "दोषः:".red().bold(), e);
            std::process::exit(1);
        }
    };

    let source = if ime {
        Transliterator::to_devanagari(&raw)
    } else {
        raw
    };

    run_code(&source);
}

fn run_code(source: &str) {
    let mut lexer = Lexer::new(source);
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

    let mut interpreter = Interpreter::new();
    interpreter.load_program(&program);
    if let Err(e) = interpreter.run_main() {
        eprintln!("{} {}", "निष्पादने दोषः (Runtime error):".red().bold(), e);
        std::process::exit(1);
    }
}

fn run_repl(mut ime_enabled: bool) {
    println!("{}", "=========================================================".cyan());
    println!("{}", "  ॥ सङ्स्कृ सङ्वादकम् ॥ (Sanskipt v०.१.०)".yellow().bold());
    println!("{}", "  Python-Style Dynamic Scripting Shell for Sanskrit");
    println!("  Type code directly. Toggle phonetic typing with ':ime'.");
    println!("  विरामार्थं 'विराम', 'exit', अथवा Ctrl+D लिखन्तु।");
    println!("{}", "=========================================================".cyan());

    let mut rl = rustyline::DefaultEditor::new().expect("Failed to initialize readline");
    let mut interpreter = Interpreter::new();

    loop {
        let prompt = if ime_enabled {
            "॥ सङ्स्कृ [IME] ॥ >>> ".green().to_string()
        } else {
            "॥ सङ्स्कृ ॥ >>> ".yellow().to_string()
        };

        match rl.readline(&prompt) {
            Ok(line) => {
                let trimmed = line.trim();
                if trimmed == "विराम" || trimmed == "exit" || trimmed == "quit" {
                    println!("{}", "सङ्वादकः समाप्तः। (Goodbye!)".cyan());
                    break;
                }
                if trimmed == ":ime" {
                    ime_enabled = !ime_enabled;
                    println!(
                        "Phonetic typing (IME): {}",
                        if ime_enabled { "ON (सक्रिय)".green() } else { "OFF (निष्क्रिय)".red() }
                    );
                    continue;
                }
                if trimmed.is_empty() {
                    continue;
                }

                let _ = rl.add_history_entry(line.as_str());

                // Transliterate if IME enabled
                let actual_line = if ime_enabled {
                    Transliterator::to_devanagari(trimmed)
                } else {
                    trimmed.to_string()
                };

                // Add Danda if line doesn't end with Danda or Iti
                let with_danda = if !actual_line.ends_with('।')
                    && !actual_line.ends_with('॥')
                    && !actual_line.ends_with("इति")
                    && !actual_line.is_empty()
                {
                    format!("{}।", actual_line)
                } else {
                    actual_line
                };

                let mut lexer = Lexer::new(&with_danda);
                match lexer.tokenize() {
                    Ok(tokens) => {
                        let mut parser = Parser::new(tokens);
                        match parser.parse_program() {
                            Ok(prog) => {
                                // Execute program incrementally into persistent global environment
                                for item in prog.items {
                                    match item {
                                        sankode_core::TopLevelItem::Function(func) => {
                                            println!("{} {}", "क्रिया संदृष्टा:".green(), func.name);
                                            interpreter.functions.insert(func.name.clone(), func);
                                        }
                                        sankode_core::TopLevelItem::Struct(s) => {
                                            println!("{} {}", "संरचना संदृष्टा:".green(), s.name);
                                            interpreter.structs.insert(s.name.clone(), s);
                                        }
                                        sankode_core::TopLevelItem::Impl(imp) => {
                                            println!("{} {}", "विधानं संदृष्टम्:".green(), imp.target);
                                            for method in imp.methods {
                                                interpreter.methods.insert(
                                                    (imp.target.clone(), method.name.clone()),
                                                    method,
                                                );
                                            }
                                        }
                                        sankode_core::TopLevelItem::Statement(stmt) => {
                                            match interpreter.eval_statement(&stmt) {
                                                Ok(Some(val)) => {
                                                    if val != Value::Unit {
                                                        println!("{}", val.display_devanagari().cyan().bold());
                                                    }
                                                }
                                                Ok(None) => {}
                                                Err(e) => eprintln!("{} {}", "दोषः:".red(), e),
                                            }
                                        }
                                        sankode_core::TopLevelItem::Comment(_) => {}
                                    }
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
