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

    /// Run with native AOT compilation for maximum speed
    #[arg(long)]
    native: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Execute a Sankode source file (.सङ् / .सङ्स्कृत्)
    Run {
        #[arg(value_name = "FILE")]
        path: PathBuf,
        /// Compile to native binary and execute directly for maximum speed
        #[arg(long)]
        native: bool,
    },
    /// Compile a Sankode source file to an optimized native binary via C99 AOT compiler
    Build {
        #[arg(value_name = "FILE")]
        path: PathBuf,
        /// Output executable path (default: same name as source without extension or .exe)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Emit C code instead of building binary
        #[arg(long)]
        emit_c: bool,
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
    /// Launch the Sankode Studio IDE (सङ्कोड वेधशाला)
    Studio {
        #[arg(short, long, default_value_t = 4040)]
        port: u16,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Run { path, native }) => {
            if native || cli.native {
                run_native(&path);
            } else {
                run_file(&path);
            }
        }
        Some(Commands::Build { path, output, emit_c }) => build_file(&path, output, emit_c),
        Some(Commands::Check { path }) => check_file(&path),
        Some(Commands::Tokens { path }) => print_tokens(&path),
        Some(Commands::Parse { path }) => print_ast(&path),
        Some(Commands::Repl) => run_repl(),
        Some(Commands::Studio { port }) => launch_studio(port),
        None => {
            if let Some(file) = cli.file {
                if cli.native {
                    run_native(&file);
                } else {
                    run_file(&file);
                }
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

fn run_native(path: &PathBuf) {
    let program = load_and_verify(path);
    let ext = if cfg!(windows) { "exe" } else { "bin" };
    let temp_exe = std::env::temp_dir().join(format!("sankode_tmp_{}.{}", std::process::id(), ext));
    
    if let Err(e) = sankode_codegen::compile_program_to_binary(&program, &temp_exe) {
        eprintln!("{} {}", "सङ्कलने दोषः (AOT compilation error):".red().bold(), e);
        std::process::exit(1);
    }

    let status = std::process::Command::new(&temp_exe).status();
    let _ = std::fs::remove_file(&temp_exe);
    if let Ok(st) = status {
        if !st.success() {
            std::process::exit(st.code().unwrap_or(1));
        }
    } else if let Err(e) = status {
        eprintln!("{} {}", "निष्पादने दोषः (Execution error):".red().bold(), e);
        std::process::exit(1);
    }
}

fn build_file(path: &PathBuf, output: Option<PathBuf>, emit_c: bool) {
    let program = load_and_verify(path);
    let c_code = match sankode_codegen::generate_c_source(&program) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{} {}", "सङ्कलने दोषः (Codegen error):".red().bold(), e);
            std::process::exit(1);
        }
    };

    if emit_c {
        let out_path = output.unwrap_or_else(|| path.with_extension("c"));
        if let Err(e) = fs::write(&out_path, c_code) {
            eprintln!("{} संचिकालेखने दोषः: {}", "दोषः:".red().bold(), e);
            std::process::exit(1);
        }
        println!("{} C source emitted to: {}", "✓".green().bold(), out_path.display().to_string().cyan());
        return;
    }

    let default_out = if cfg!(windows) {
        path.with_extension("exe")
    } else {
        path.with_extension("")
    };
    let out_path = output.unwrap_or(default_out);
    println!("{} Compiling {} to native machine binary...", "॥ सङ्कोड ॥".cyan().bold(), path.display());
    match sankode_codegen::compile_to_binary(&c_code, &out_path, None) {
        Ok(()) => {
            println!("{} Native binary built successfully: {}", "✓".green().bold(), out_path.display().to_string().green());
        }
        Err(e) => {
            eprintln!("{} {}", "सङ्कलने दोषः (AOT compilation error):".red().bold(), e);
            std::process::exit(1);
        }
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

fn launch_studio(port: u16) {
    let mut cmd = std::process::Command::new("cargo");
    cmd.args(["run", "-p", "sankode-studio", "--", "--port", &port.to_string()]);
    let status = cmd.status();
    if let Err(e) = status {
        eprintln!("Failed to launch Sankode Studio: {}", e);
    }
}
