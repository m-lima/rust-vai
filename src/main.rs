mod error;
mod executors;
//mod support;

//fn support_mode() -> Result<(), String> {
//    match std::env::args().nth(2) {
//        Some(arg) => match arg.as_str() {
//            "r" => executors::load_from_stdin().map(|e| e.to_json()),
//            _ => (),
//        },
//        None => Err("No command given"),
//    }
//}

fn main() {
    if let Err(err) = if match std::env::args().nth(1) {
        Some(arg) => "-" == arg,
        None => false,
    } {
        executors::load_default().map(|_| ())
    } else {
        executors::load_default().map(|_| ())
    } {
        eprintln!("{}", err);
    }
}
