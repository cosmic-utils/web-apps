use cef::*;

fn main() {
    let args = args::Args::new();
    let _ = execute_process(Some(args.as_main_args()), None, std::ptr::null_mut());
}
