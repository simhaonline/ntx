use ntx::cli::CliHandler;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    match CliHandler::new() {
        Ok(handler) => handler.dispatch(&args),
        Err(e) => {
            eprintln!("Failed to initialize Nityaksha Engine: {}", e);
            std::process::exit(1);
        }
    }
}
