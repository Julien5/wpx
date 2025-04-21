use std::process::Command;

pub fn run() {
    Command::new("/opt/typst/typst-x86_64-unknown-linux-musl/typst")
        .arg("compile")
        .arg("--root")
        .arg("/")
        .arg("/tmp/test.typ")
        .output()
        .expect("typst compiler failed");
    println!("pdf is done");
}
