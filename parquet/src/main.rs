use std::{fs::File, io::Read};

fn main() {
    let mut f = File::open("data/data.parquet").unwrap();
    let mut buf = [0; 512];
    f.read_exact(&mut buf[..4]).unwrap();

    println!("{:?}", &buf[..4]);
    println!("{:?}", b"PAR1");
}
