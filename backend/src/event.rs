pub trait Sender {
    fn send(&mut self, data: &String);
}

pub type SenderHandler = Box<dyn Sender + Send + Sync>;
pub type SenderHandlerLock = std::sync::RwLock<Option<SenderHandler>>;

#[cfg(not(target_arch = "wasm32"))]
pub async fn send_worker(handler: &SenderHandlerLock, data: &String) {
    let _ = handler.write().unwrap().as_mut().unwrap().send(&data);
}

#[cfg(target_arch = "wasm32")]
pub async fn send_worker(handler: &SenderHandlerLock, data: &String) {
    let _ = handler.write().unwrap().as_mut().unwrap().send(&data);
    let tick = std::time::Duration::from_millis(0);
    let _ = wasmtimer::tokio::sleep(tick).await;
}

#[derive(Clone)]
pub struct ConsoleEventSender {}

impl Sender for ConsoleEventSender {
    fn send(&mut self, data: &String) {
        log::info!("event: {}", data);
    }
}
