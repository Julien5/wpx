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
        .try_init();
}

#[cfg(target_arch = "wasm32")]
fn setup_log() {
    println!("init logger not needed in browser (i dont know why)");
}

pub fn setup() {
    setup_log();
}
