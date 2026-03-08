mod color;
mod segments;

use std::env;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.get(1).map(String::as_str) {
        Some("path") => {
            let home = env::var("HOME").unwrap_or_default();
            let pwd = env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();
            let max_dir_size = args.get(2).and_then(|s| s.parse::<usize>().ok());
            print!("{}", segments::path::render(&home, &pwd, max_dir_size));
        }
        Some("git") => print!("{}", segments::git::render(Path::new("."))),
        Some("nix-shell") => print!("{}", segments::nix_shell::render()),
        Some("prompt") => {
            let max_dir_size = args.get(2).and_then(|s| s.parse::<usize>().ok());
            let mut ctx = segments::prompt::PromptContext::gather(max_dir_size);
            print!("{}", segments::prompt::render(&mut ctx));
        }
        Some("tmux-title") => {
            let home = env::var("HOME").unwrap_or_default();
            let pwd = env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();
            println!("{}", segments::tmux_title::render(&home, &pwd));
        }
        _ => {
            eprintln!("Usage: plx <path|git|nix-shell|prompt|tmux-title>");
            std::process::exit(1);
        }
    }
}
