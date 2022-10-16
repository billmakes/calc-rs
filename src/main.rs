use calc::repl;

fn main() {
    if let Err(e) = repl() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
