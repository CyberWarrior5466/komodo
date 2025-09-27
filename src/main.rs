use std::io::{self, Read};
use tempfile::NamedTempFile;

fn main() {
    let path = get_file_path();

    std::process::Command::new("arm-linux-gnueabi-as");
    dbg!(path);
}

fn get_file_path() -> String {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        // get file path from cli args
        let path = args[1].clone();
        return path;
    } else {
        // return a path of a tempfile from stdin
        let mut buf = String::new();
        io::stdin().lock().read_to_string(&mut buf).unwrap();
        let file = NamedTempFile::new().unwrap();
        let path = file.path();
        return path.to_string_lossy().into_owned();
    }
}
