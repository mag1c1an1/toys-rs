use std::env;


mod open_file;
mod process;
mod ps_utils;


fn main() {
    let args:Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} <name or pid of target>.",args[0]);
        std::process::exit(1);
    }
    #[allow(unused)]
    let target = &args[1];
    unimplemented!()
}
