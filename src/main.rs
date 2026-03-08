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
            print!("{}", segments::path::render(&home, &pwd));
        }
        Some("git") => print!("{}", segments::git::render(Path::new("."))),
        Some("tmux-title") => {
            let home = env::var("HOME").unwrap_or_default();
            let pwd = env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();
            println!("{}", segments::tmux_title::render(&home, &pwd));
        }
        _ => {
            eprintln!("Usage: plx <path|git|tmux-title>");
            std::process::exit(1);
        }
    }
}
