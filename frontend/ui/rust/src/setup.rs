#[cfg(not(target_arch = "wasm32"))]
fn setup_log() {
    println!("init logger");
    use std::io::Write;
    let _ = env_logger::Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] - {}",
                chrono::Local::now().format("%H:%M:%S:%f"),
                record.level(),
                record.args()
            )
        })
        .filter_level(log::LevelFilter::Trace)
        //.filter_level(log::LevelFilter::Off)
        .try_init();
}

#[cfg(target_arch = "wasm32")]
fn setup_log() {
    println!("init logger not needed in browser (i dont know why)");
}

pub fn setup() {
    setup_log();
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
}
