mod html;

use clap::Parser as ClapParser;
use colored::Colorize;
use html::STUDIO_HTML;
use sankode_borrowck::BorrowChecker;
use sankode_eval::Interpreter;
use sankode_ime::Transliterator;
use sankode_lexer::Lexer;
use sankode_parser::Parser;
use sankode_semantics::TypeChecker;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

#[derive(ClapParser)]
#[command(name = "sankode-studio")]
#[command(author = "tovganesh")]
#[command(version = "0.1.0")]
#[command(about = "॥ सङ्कोड वेधशाला ॥ - Minimal IDE for Sankode and Sanskipt", long_about = None)]
struct Cli {
    /// Port to bind the IDE web server to
    #[arg(short, long, default_value_t = 4040)]
    port: u16,

    /// Do not automatically open browser
    #[arg(long)]
    no_open: bool,
}

#[derive(Deserialize)]
struct RunRequest {
    code: String,
    mode: Option<String>,
}

#[derive(Serialize)]
struct RunResponse {
    success: bool,
    output: String,
    error: Option<String>,
    tokens: String,
    ast: String,
}

#[derive(Deserialize)]
struct CheckRequest {
    code: String,
}

#[derive(Serialize)]
struct CheckResponse {
    valid: bool,
    error: Option<String>,
}

#[derive(Deserialize)]
struct TransliterateRequest {
    text: String,
}

#[derive(Serialize)]
struct TransliterateResponse {
    devanagari: String,
}

fn main() {
    let cli = Cli::parse();
    let addr = format!("127.0.0.1:{}", cli.port);

    let listener = match TcpListener::bind(&addr) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("{} Failed to bind server to {}: {}", "दोषः:".red().bold(), addr, e);
            std::process::exit(1);
        }
    };

    println!("{}", "=========================================================".cyan());
    println!("{}", "  ॥ सङ्कोड वेधशाला ॥ (Sankode Studio v०.१.०)".yellow().bold());
    println!("  Minimal Devanagari IDE for Sankode & Sanskipt");
    println!("  सङ्केतवेधशाला अत्र उपलब्धा अस्ति:");
    println!("  {}", format!("http://{}", addr).green().bold());
    println!("  विरामार्थं Ctrl+C नुदन्तु। (Press Ctrl+C to stop)");
    println!("{}", "=========================================================".cyan());

    if !cli.no_open {
        open_browser(&format!("http://{}", addr));
    }

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                std::thread::spawn(move || {
                    handle_client(stream);
                });
            }
            Err(e) => eprintln!("Connection error: {}", e),
        }
    }
}

const MAX_PAYLOAD_SIZE: usize = 1024 * 1024; // 1 MB limit

fn handle_client(mut stream: TcpStream) {
    let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(5)));
    let _ = stream.set_write_timeout(Some(std::time::Duration::from_secs(5)));

    let mut buffer = Vec::new();
    let mut chunk = [0; 4096];
    let mut content_length = 0;
    let mut header_end = None;

    loop {
        let n = match stream.read(&mut chunk) {
            Ok(n) if n > 0 => n,
            _ => break,
        };
        buffer.extend_from_slice(&chunk[..n]);

        if buffer.len() > MAX_PAYLOAD_SIZE {
            let too_large = "HTTP/1.1 413 Payload Too Large\r\nContent-Length: 0\r\n\r\n";
            let _ = stream.write_all(too_large.as_bytes());
            return;
        }

        if header_end.is_none() {
            if let Some(pos) = buffer.windows(4).position(|w| w == b"\r\n\r\n") {
                header_end = Some(pos + 4);
                let header_str = String::from_utf8_lossy(&buffer[..pos]);
                for line in header_str.lines() {
                    let lower = line.to_lowercase();
                    if lower.starts_with("content-length:") {
                        let parts: Vec<&str> = line.split(':').collect();
                        if parts.len() >= 2 {
                            content_length = parts[1].trim().parse().unwrap_or(0);
                        }
                    }
                }
                if content_length > MAX_PAYLOAD_SIZE {
                    let too_large = "HTTP/1.1 413 Payload Too Large\r\nContent-Length: 0\r\n\r\n";
                    let _ = stream.write_all(too_large.as_bytes());
                    return;
                }
            }
        }

        if let Some(hdr_len) = header_end {
            if buffer.len() >= hdr_len + content_length {
                break;
            }
        }
    }

    let hdr_len = match header_end {
        Some(len) => len,
        None => return,
    };

    let header_str = String::from_utf8_lossy(&buffer[..hdr_len]);
    let mut lines = header_str.lines();
    let request_line = match lines.next() {
        Some(l) => l,
        None => return,
    };

    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return;
    }

    let method = parts[0];
    let path = parts[1];

    if method == "GET" && (path == "/" || path == "/index.html") {
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}",
            STUDIO_HTML.len(),
            STUDIO_HTML
        );
        let _ = stream.write_all(response.as_bytes());
        return;
    }

    let body = String::from_utf8_lossy(&buffer[hdr_len..hdr_len + content_length]);

    match (method, path) {
        ("POST", "/api/run") => {
            let req: RunRequest = match serde_json::from_str(&body) {
                Ok(r) => r,
                Err(e) => {
                    send_json(&mut stream, 400, &serde_json::json!({ "error": e.to_string() }));
                    return;
                }
            };
            let mode = req.mode.as_deref().unwrap_or("sankode");
            let resp = execute_code_for_studio(&req.code, mode);
            send_json(&mut stream, 200, &resp);
        }
        ("POST", "/api/check") => {
            let req: CheckRequest = match serde_json::from_str(&body) {
                Ok(r) => r,
                Err(e) => {
                    send_json(&mut stream, 400, &serde_json::json!({ "error": e.to_string() }));
                    return;
                }
            };
            let resp = check_code_for_studio(&req.code);
            send_json(&mut stream, 200, &resp);
        }
        ("POST", "/api/transliterate") => {
            let req: TransliterateRequest = match serde_json::from_str(&body) {
                Ok(r) => r,
                Err(e) => {
                    send_json(&mut stream, 400, &serde_json::json!({ "error": e.to_string() }));
                    return;
                }
            };
            let dev = Transliterator::to_devanagari(&req.text);
            send_json(&mut stream, 200, &TransliterateResponse { devanagari: dev });
        }
        _ => {
            let not_found = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n";
            let _ = stream.write_all(not_found.as_bytes());
        }
    }
}

fn send_json<T: Serialize>(stream: &mut TcpStream, status: u16, data: &T) {
    let json = serde_json::to_string(data).unwrap_or_default();
    let status_line = match status {
        200 => "200 OK",
        400 => "400 Bad Request",
        _ => "500 Internal Server Error",
    };
    let response = format!(
        "HTTP/1.1 {}\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}",
        status_line,
        json.len(),
        json
    );
    let _ = stream.write_all(response.as_bytes());
}

fn execute_code_for_studio(code: &str, mode: &str) -> RunResponse {
    let mut lexer = Lexer::new(code);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => {
            return RunResponse {
                success: false,
                output: String::new(),
                error: Some(format!("पदच्छेदे दोषः (Lexer error): {}", e)),
                tokens: String::new(),
                ast: String::new(),
            };
        }
    };

    let tokens_debug: String = tokens
        .iter()
        .map(|t| format!("{:15} {:?} at {}\n", t.kind.to_string(), t.kind, t.span))
        .collect();

    let mut parser = Parser::new(tokens);
    let program = match parser.parse_program() {
        Ok(p) => p,
        Err(e) => {
            return RunResponse {
                success: false,
                output: String::new(),
                error: Some(format!("वाक्यरचनादोषः (Parser error): {}", e)),
                tokens: tokens_debug,
                ast: String::new(),
            };
        }
    };

    let ast_debug = format!("{:#?}", program);

    if mode == "sankode" {
        let mut type_checker = TypeChecker::new();
        if let Err(e) = type_checker.check_program(&program) {
            return RunResponse {
                success: false,
                output: String::new(),
                error: Some(format!("प्रकारदोषः (Type error): {}", e)),
                tokens: tokens_debug,
                ast: ast_debug,
            };
        }

        let mut borrow_checker = BorrowChecker::new();
        if let Err(e) = borrow_checker.check_program(&program) {
            return RunResponse {
                success: false,
                output: String::new(),
                error: Some(format!("स्वामित्वदोषः (Borrow error): {}", e)),
                tokens: tokens_debug,
                ast: ast_debug,
            };
        }
    }

    let mut interpreter = Interpreter::new();
    interpreter.max_steps = Some(500_000);
    interpreter.max_call_depth = 300;
    interpreter.stdout_capture = Some(Vec::new());
    interpreter.load_program(&program);

    match interpreter.run_main() {
        Ok(_) => {
            let captured = interpreter.stdout_capture.unwrap_or_default();
            RunResponse {
                success: true,
                output: captured.join("\n"),
                error: None,
                tokens: tokens_debug,
                ast: ast_debug,
            }
        }
        Err(e) => {
            let captured = interpreter.stdout_capture.unwrap_or_default();
            RunResponse {
                success: false,
                output: captured.join("\n"),
                error: Some(format!("निष्पादने दोषः (Runtime error): {}", e)),
                tokens: tokens_debug,
                ast: ast_debug,
            }
        }
    }
}

fn check_code_for_studio(code: &str) -> CheckResponse {
    let mut lexer = Lexer::new(code);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => {
            return CheckResponse {
                valid: false,
                error: Some(format!("पदच्छेदे दोषः: {}", e)),
            };
        }
    };

    let mut parser = Parser::new(tokens);
    let program = match parser.parse_program() {
        Ok(p) => p,
        Err(e) => {
            return CheckResponse {
                valid: false,
                error: Some(format!("वाक्यरचनादोषः: {}", e)),
            };
        }
    };

    let mut type_checker = TypeChecker::new();
    if let Err(e) = type_checker.check_program(&program) {
        return CheckResponse {
            valid: false,
            error: Some(format!("प्रकारदोषः: {}", e)),
        };
    }

    let mut borrow_checker = BorrowChecker::new();
    if let Err(e) = borrow_checker.check_program(&program) {
        return CheckResponse {
            valid: false,
            error: Some(format!("स्वामित्वदोषः: {}", e)),
        };
    }

    CheckResponse {
        valid: true,
        error: None,
    }
}

fn open_browser(url: &str) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", url])
            .spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(url).spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    }
}
