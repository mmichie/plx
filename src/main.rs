mod color;
mod segments;
mod shell;

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
            let max_dir_size = args.get(2).and_then(|s| s.parse().ok());
            let exit_status = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(0);
            let duration_ms = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(0);
            let job_count = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(0);
            let mut ctx = segments::prompt::PromptContext::gather(
                max_dir_size,
                exit_status,
                duration_ms,
                job_count,
            );
            print!("{}", color::zsh_wrap_escapes(&segments::prompt::render(&mut ctx)));
        }
        Some("tmux-title") => {
            let home = env::var("HOME").unwrap_or_default();
            let pwd = env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();
            println!("{}", segments::tmux_title::render(&home, &pwd));
        }
        Some("init") => {
            if let Some("zsh") = args.get(2).map(String::as_str) {
                print!("{}", shell::init_zsh());
            } else {
                eprintln!("Usage: plx init <zsh>");
                std::process::exit(1);
            }
        }
        _ => {
            eprintln!("Usage: plx <path|git|nix-shell|prompt|tmux-title|init>");
            std::process::exit(1);
        }
    }
}
