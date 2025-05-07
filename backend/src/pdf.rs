use std::process::Command;

pub fn run(input: &str, output: &str) -> () {
    let mut cmd = Command::new("/opt/typst/typst-x86_64-unknown-linux-musl/typst");
    cmd.arg("compile");
    cmd.arg("--root");
    cmd.arg("/");
    cmd.arg(input);
    cmd.arg(output);
    cmd.output().expect("typst compiler failed");
    println!("pdf is done");
}
