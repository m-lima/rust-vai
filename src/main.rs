//#![deny(warnings)]
#![warn(rust_2018_idioms)]

mod completer;
mod executors;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Clone)]
struct VaiError(&'static str);
impl std::error::Error for VaiError {}
impl std::fmt::Display for VaiError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(fmt, "{}", self.0)
    }
}

//fn extract_query(index: usize) -> Result<String> {
//    if std::env::args().len() <= index {
//        Err(error::new("main::extract_query", "No query specified").into())
//    } else {
//        Ok(std::env::args()
//            .skip(index)
//            .collect::<Vec<String>>()
//            .join(" "))
//    }
//}
//
//fn support() -> Result {
//    match std::env::args().nth(2) {
//        Some(arg) => match arg.as_str() {
//            //            "r" => executors::load_from_stdin()?.save_default(),
//            //            "w" => executors::load_default()?.to_json(),
//            //            "t" => executors::load_default()?.list_targets(),
//            //            "s" => match std::env::args().nth(3) {
//            //                Some(target) => executors::load_default()?
//            //                    .find(target)?
//            //                    .suggest(extract_query(4)?),
//            //                None => Err(error::new("main::support", "No target provided")),
//            //            },
//            cmd => {
//                Err(error::new("main::support", format!("Command not recognized: {}", cmd)).into())
//            }
//        },
//        None => Err(error::new("main::support", "No command given").into()),
//    }
//}
//
//fn execute() -> Result {
//    match std::env::args().nth(1) {
//        Some(target) => executors::load_default()?
//            .find(target)?
//            .execute(extract_query(2)?),
//        None => Err(error::new("main::execute", "Invalid target specified").into()),
//    }
//}

fn main() -> Result {
    Err(VaiError("Expected arguments").into())
    //    if "-" == std::env::args().nth(1).ok_or(VaiError("Expected arguments"))? {
    //        support()
    //    } else {
    //        execute()
    //    }
}
