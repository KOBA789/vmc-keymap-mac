use cacao::macos::{App, AppDelegate};
use core_graphics::{
    event::{CGEvent, CGEventTapLocation, CGKeyCode, KeyCode},
    event_source::{CGEventSource, CGEventSourceStateID},
};

mod osc;

#[derive(Default)]
struct BasicApp {}

impl AppDelegate for BasicApp {
    fn did_finish_launching(&self) {
        std::thread::spawn(main_loop);
    }
}

struct DropHook;
impl Drop for DropHook {
    fn drop(&mut self) {
        std::process::exit(0);
    }
}

fn main_loop() -> ! {
    let _hook = DropHook;

    let mut buf = vec![0u8; 65535];
    let socket = std::net::UdpSocket::bind((std::net::Ipv4Addr::UNSPECIFIED, 39600)).unwrap();
    loop {
        let len = socket.recv(&mut buf).unwrap();
        handle_osc_packet(&buf[..len]).ok();
    }
}

fn handle_osc_packet(packet_bytes: &[u8]) -> Result<(), ()> {
    let mut packet = osc::packet::Parser::new(packet_bytes)?;
    while !packet.is_end_of_data() {
        let mut message = packet.read_message()?;
        if message.address() != b"/VMC/Ext/Con" {
            continue;
        }
        if message.num_of_rest_arguments() != 8 {
            continue;
        }
        let active = message.read_argument()?.as_int32().ok_or(())?;
        let name = message.read_argument()?.as_string().ok_or(())?;
        let is_left = message.read_argument()?.as_int32().ok_or(())?;
        let _is_touch = message.read_argument()?.as_int32().ok_or(())?;
        let _is_axis = message.read_argument()?.as_int32().ok_or(())?;
        let _axis_x = message.read_argument()?.as_float32().ok_or(())?;
        let _axis_y = message.read_argument()?.as_float32().ok_or(())?;
        let _axis_z = message.read_argument()?.as_float32().ok_or(())?;
        if name == b"ClickBbutton" && is_left == 0 && active == 1 {
            post_key_event(KeyCode::RIGHT_ARROW);
        }
        if name == b"ClickAbutton" && is_left == 0 && active == 1 {
            post_key_event(KeyCode::LEFT_ARROW);
        }
    }
    Ok(())
}

fn post_key_event(keycode: CGKeyCode) {
    let event_source = CGEventSource::new(CGEventSourceStateID::HIDSystemState).unwrap();
    let keydown_event = CGEvent::new_keyboard_event(event_source.clone(), keycode, true).unwrap();
    let keyup_event = CGEvent::new_keyboard_event(event_source, keycode, false).unwrap();
    let tap_location = CGEventTapLocation::HID;
    keydown_event.post(tap_location);
    keyup_event.post(tap_location);
}

fn main() {
    App::new("com.koba789.vmc-keymap", BasicApp::default()).run();
}
