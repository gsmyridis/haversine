use std::fs::File;
use std::io::Read;

mod parse;
use parse::Cursor;

fn main() {
    let file_name = "../gendata/pairs.json";
    let mut file = File::open(file_name).expect("Failed to open file");
    let mut buffer = Vec::new();
    let n = file.read_to_end(&mut buffer).expect("Failed to read file");
    println!("Read {n} bytes: {buffer:?}");

    // let mut cursor = Cursor::new(buffer.iter());
    // cursor.next_value();
}
