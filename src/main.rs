use std::process;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(e) = bitcoin_analyser::start(args) {
        eprintln!("Something went wrong, context: {}", e);
        process::exit(1);
    }
}
