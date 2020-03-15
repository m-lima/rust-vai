use std::io::Read;
use std::io::Write;

use colored::Colorize;

pub(super) fn run(name: String) -> ! {
    print!("{}> ", name.blue());
    let _ = std::io::stdout().flush();

    let mut buffer = [0u8; 4];

    match std::io::stdin().read(&mut buffer) {
        Ok(bytes) => println!("Read {} byte(s): {}", bytes, buffer[0]),
        Err(err) => eprintln!("Error: {}", err),
    }

    std::process::exit(0);
}
