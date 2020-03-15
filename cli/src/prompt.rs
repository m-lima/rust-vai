pub(super) fn run() -> ! {
    std::thread::sleep(std::time::Duration::from_secs(10));
    match std::fs::write(std::path::Path::new("/tmp/vai_output"), format!("{}", atty::is(atty::Stream::Stdout))) {
        Ok(_) => std::process::exit(0),
        Err(_) => std::process::exit(-1),
    }
}
