mod cli;
mod parser;
mod tray;

use tray::NordLayerTray;

fn main() {
    let service = ksni::TrayService::new(NordLayerTray::new());
    let _handle = service.handle();
    service.spawn();

    loop {
        std::thread::park();
    }
}
