use std::fs::File;
use std::io::Read;

mod parse;
use parse::Cursor;

fn main() {
    let file_name = "../gendata/pairs.json";
    let mut file = File::open(file_name).expect("Failed to open file");
    let mut string = String::new();
    let _n = file
        .read_to_string(&mut string)
        .expect("Failed to read file");

    let json = Cursor::new(&string).parse().unwrap();
    println!("{json:?}");

    // let mut cursor = Cursor::new(buffer.iter());
    // cursor.next_value();
}
