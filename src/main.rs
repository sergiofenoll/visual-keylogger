use std::os::unix::prelude::OsStrExt;

use notify::Watcher;

fn handle_event(event: evdev::InputEvent) {
    match event.event_type() {
        evdev::EventType::KEY => log::debug!("{:?}", event),
        _ => return,
    }
}

fn spawn_thread(mut device: evdev::Device) {
    let device_path = device.physical_path().unwrap().to_owned();
    log::debug!(
        "Spawning thread to handle events from device {}",
        device_path
    );
    let join_handle = std::thread::spawn(move || loop {
        let device_path = device.physical_path().unwrap().to_owned();
        match device.fetch_events() {
            Ok(events) => {
                for event in events {
                    handle_event(event);
                }
            }
            Err(e) if e.to_string().contains("No such device") => {
                break;
            }
            Err(e) => {
                log::error!(
                    "Error fetching events from device {:?}: {:?}",
                    device_path,
                    e
                );
                break;
            }
        }
    });
    match join_handle.join() {
        Ok(_) => log::debug!(
            "Finished thread to handle events from device {}",
            device_path
        ),
        Err(e) => log::error!(
            "Thread handling events for device {} panicked: {:?}",
            device_path,
            e
        ),
    }
}

fn main() {
    env_logger::Builder::new()
        .filter(None, log::LevelFilter::Debug)
        .init();

    let input_devices_path = "/dev/input/by-id";

    let mut paths = std::fs::read_dir(input_devices_path).unwrap();

    while let Some(Ok(path)) = paths.next() {
        if !path.file_name().as_bytes().ends_with(b"-event-kbd") {
            continue;
        }

        match evdev::Device::open(path.path()) {
            Ok(device) => {
                spawn_thread(device);
            }
            Err(e) => log::error!("Error creating device {}", e),
        }
    }

    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = notify::watcher(tx, std::time::Duration::from_millis(100)).unwrap();
    watcher
        .watch(input_devices_path, notify::RecursiveMode::NonRecursive)
        .unwrap();
    loop {
        match rx.recv() {
            Ok(event) => {
                log::debug!("{:?}", event);
                match event {
                    notify::DebouncedEvent::Create(path) => {
                        if !path.as_os_str().as_bytes().ends_with(b"-event-kbd") {
                            continue;
                        }
                        let device = evdev::Device::open(path).unwrap();
                        spawn_thread(device);
                    }
                    _ => continue,
                }
            }
            Err(e) => log::error!("{:?}", e),
        }
    }
}
