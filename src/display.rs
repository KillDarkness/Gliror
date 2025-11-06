use colored::*;
use indicatif::{ProgressBar, ProgressStyle};

pub fn print_cyan_ascii() {
    let ascii_art = r#"
  ▄████  ██▓     ██▓▄▄▄█████▓ ▒█████   ██▀███  
 ██▒ ▀█▒▓██▒    ▓██▒▓  ██▒ ▓▒▒██▒  ██▒▓██ ▒ ██▒
▒██░▄▄▄░▒██░    ▒██▒▒ ▓██░ ▒░▒██░  ██▒▓██ ░▄█ ▒
░▓█  ██▓▒██░    ░██░░ ▓██▓ ░ ▒██   ██░▒██▀▀█▄  
░▒▓███▀▒░██████▒░██░  ▒██▒ ░ ░ ████▓▒░░██▓ ▒██▒
 ░▒   ▒ ░ ▒░▓  ░░▓    ▒ ░░   ░ ▒░▒░▒░ ░ ▒▓ ░▒▓░
  ░   ░ ░ ░ ▒  ░ ▒ ░    ░      ░ ▒ ▒░   ░▒ ░ ▒░
░ ░   ░   ░ ░    ▒ ░  ░      ░ ░ ░ ▒    ░░   ░ 
      ░     ░  ░ ░             ░ ░     ░     
    "#;
    
    println!("{}", ascii_art.cyan());
}

pub fn setup_progress_bar() -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap()
    );
    pb
}