/// ANSI color codes
pub struct Colors;

impl Colors {
    pub const RED: &'static str = "\x1b[31m";
    pub const GREEN: &'static str = "\x1b[32m";
    pub const YELLOW: &'static str = "\x1b[33m";
    pub const BLUE: &'static str = "\x1b[34m";
    pub const PURPLE: &'static str = "\x1b[35m";
    pub const CYAN: &'static str = "\x1b[36m";
    pub const RESET: &'static str = "\x1b[0m";
}

/// Print colored text to stdout
pub fn print_colored(text: &str, color: &str) {
    println!("{}{}{}", color, text, Colors::RESET);
}

/// Print colored text to stderr
pub fn eprint_colored(text: &str, color: &str) {
    eprintln!("{}{}{}", color, text, Colors::RESET);
}

/// Print success message
pub fn print_success(text: &str) {
    print_colored(text, Colors::GREEN);
}

/// Print error message
pub fn print_error(text: &str) {
    eprint_colored(text, Colors::RED);
}

/// Print warning message
pub fn print_warning(text: &str) {
    print_colored(text, Colors::YELLOW);
}

/// Print info message
pub fn print_info(text: &str) {
    print_colored(text, Colors::BLUE);
}

/// Print header message
pub fn print_header(text: &str) {
    print_colored(text, Colors::PURPLE);
}

/// Show a helpful error with suggestions
pub fn show_error_with_help(error: &str, suggestions: &[&str]) {
    print_error(&format!("❌ {error}"));
    println!();
    if !suggestions.is_empty() {
        print_colored("💡 Here's how to fix it:", Colors::CYAN);
        for (i, suggestion) in suggestions.iter().enumerate() {
            println!("   {}. {}", i + 1, suggestion);
        }
        println!();
    }
}
