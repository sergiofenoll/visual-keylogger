use gdk::glib::{self, Sender};
use gtk::prelude::*;
use notify::Watcher;
use std::{os::unix::prelude::OsStrExt};

fn handle_event(event: evdev::InputEvent, sender: &Sender<String>) {
    match event.event_type() {
        evdev::EventType::KEY => {
            log::debug!("{:?}", event);
            let _ = sender.send(format!("{:?}", event.kind()));
        }
        _ => return,
    }
}

fn spawn_thread(mut device: evdev::Device, sender: Sender<String>) {
    let device_path = device.physical_path().unwrap().to_owned();
    log::debug!(
        "Spawning thread to handle events from device {}",
        device_path
    );
    std::thread::spawn(move || {
        let sender = sender;
        loop {
            let device_path = device.physical_path().unwrap().to_owned();
            match device.fetch_events() {
                Ok(events) => {
                    for event in events {
                        handle_event(event, &sender);
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
        }
    });
}

fn watch_input_devices_filesystem(input_devices_path: &str, sender: Sender<String>) {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = notify::watcher(tx, std::time::Duration::from_millis(100)).unwrap();
    watcher
        .watch(input_devices_path, notify::RecursiveMode::NonRecursive)
        .unwrap();
    std::thread::spawn(move || {
        let _move = watcher;
        let sender = sender;
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
                            spawn_thread(device, sender.clone());
                        }
                        _ => continue,
                    }
                }
                Err(e) => {
                    log::error!("{:?}", e);
                    // break;
                }
            }
        }
    });
}

fn init_event_handlers_for_existing_devices(input_devices_path: &str, sender: Sender<String>) {
    let mut paths = std::fs::read_dir(input_devices_path).unwrap();
    while let Some(Ok(path)) = paths.next() {
        if !path.file_name().as_bytes().ends_with(b"-event-kbd") {
            continue;
        }

        match evdev::Device::open(path.path()) {
            Ok(device) => {
                spawn_thread(device, sender.clone());
            }
            Err(e) => log::error!("Error creating device {}", e),
        }
    }
}

fn main() {
    env_logger::Builder::new()
        .filter(None, log::LevelFilter::Debug)
        .init();

    let input_devices_path = "/dev/input/by-id";

    let application =
        gtk::Application::new(Some("com.github.gtk-rs.examples.basic"), Default::default());
    application.connect_activate(move |app| {
        let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        init_event_handlers_for_existing_devices(input_devices_path, sender.clone());
        watch_input_devices_filesystem(input_devices_path, sender.clone());

        let window = gtk::Window::new(gtk::WindowType::Toplevel);

        window.set_application(Some(app));

        window.set_default_size(600, 80);
        window.set_position(gtk::WindowPosition::Center);
        window.set_decorated(false);
        window.set_keep_above(true);

        let vbox = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        let label = gtk::Label::new(Some("Text string"));

        vbox.add(&label);
        window.add(&vbox);

        window.show_all();

        let label_clone = label.clone();
        receiver.attach(None, move |msg: String| {
            label_clone.set_text(msg.as_str());
            glib::Continue(true)
        });
    });
    application.run();
}
