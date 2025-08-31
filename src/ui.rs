use std::io::{self, Write};

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

/// Print a separator line
pub fn print_separator() {
    println!("{}", "─".repeat(50));
}

/// Prompt user for input with a message
pub fn prompt(message: &str) -> io::Result<String> {
    print!("{}: ", message);
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    Ok(input.trim().to_string())
}

/// Prompt user for yes/no confirmation
pub fn confirm(message: &str) -> io::Result<bool> {
    loop {
        let response = prompt(&format!("{} (y/n)", message))?;
        match response.to_lowercase().as_str() {
            "y" | "yes" => return Ok(true),
            "n" | "no" => return Ok(false),
            _ => print_warning("Please enter 'y' or 'n'"),
        }
    }
}

/// Display a numbered list and get user selection
pub fn select_from_list<T: std::fmt::Display>(
    items: &[T],
    message: &str,
    allow_multiple: bool,
) -> io::Result<Vec<usize>> {
    if items.is_empty() {
        return Ok(vec![]);
    }

    println!("{}", message);
    for (i, item) in items.iter().enumerate() {
        println!("  {}. {}", i + 1, item);
    }

    if allow_multiple {
        println!("Enter numbers separated by spaces (e.g., '1 3 5'), or 'all' for all items:");
    } else {
        println!("Enter a number:");
    }

    loop {
        let input = prompt("")?;
        
        if allow_multiple && input.trim().to_lowercase() == "all" {
            return Ok((0..items.len()).collect());
        }

        let selections: Result<Vec<usize>, _> = input
            .split_whitespace()
            .map(|s| s.parse::<usize>())
            .collect();

        match selections {
            Ok(nums) => {
                let valid_nums: Vec<usize> = nums
                    .into_iter()
                    .filter_map(|n| {
                        if n > 0 && n <= items.len() {
                            Some(n - 1) // Convert to 0-based index
                        } else {
                            None
                        }
                    })
                    .collect();

                if valid_nums.is_empty() {
                    print_warning("No valid selections. Please try again.");
                } else if !allow_multiple && valid_nums.len() > 1 {
                    print_warning("Please select only one item.");
                } else {
                    return Ok(valid_nums);
                }
            }
            Err(_) => {
                print_warning("Invalid input. Please enter numbers only.");
            }
        }
    }
}

/// Display progress indicator
pub struct ProgressIndicator {
    message: String,
    step: usize,
    total: usize,
}

impl ProgressIndicator {
    pub fn new(message: String, total: usize) -> Self {
        Self {
            message,
            step: 0,
            total,
        }
    }

    pub fn step(&mut self, step_message: &str) {
        self.step += 1;
        println!(
            "[{}/{}] {}: {}",
            self.step, self.total, self.message, step_message
        );
    }

    pub fn finish(&self, final_message: &str) {
        print_success(&format!("✓ {}: {}", self.message, final_message));
    }
}
