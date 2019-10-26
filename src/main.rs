mod executors;
mod support;

fn main() {
    if match std::env::args().nth(1) {
        Some(arg) => "-" == arg,
        None => false,
    } {
        support::write_providers();
    } else {
        if let Err(err) = executors::load_default() {
            eprintln!("{}", err);
        }
    }
}
