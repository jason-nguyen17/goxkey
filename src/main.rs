mod input;
mod platform;
mod ui;

use druid::{AppLauncher, ExtEventSink, Target, WindowDesc};
use input::INPUT_STATE;
use log::debug;
use once_cell::sync::OnceCell;
use platform::{
    run_event_listener, send_backspace, send_string, Handle, KeyModifier, KEY_DELETE, KEY_ENTER,
    KEY_ESCAPE, KEY_SPACE, KEY_TAB,
};
use std::thread;
use ui::{UIDataAdapter, UPDATE_UI};

static UI_EVENT_SINK: OnceCell<ExtEventSink> = OnceCell::new();

fn process_character(handle: Handle, c: char, modifiers: KeyModifier) -> bool {
    unsafe {
        if modifiers.is_super() || modifiers.is_control() || modifiers.is_alt() {
            INPUT_STATE.new_word();
        } else if INPUT_STATE.is_tracking() {
            INPUT_STATE.push(if modifiers.is_shift() {
                c.to_ascii_uppercase()
            } else {
                c
            });
            if INPUT_STATE.should_transform_keys(&c) {
                let output = INPUT_STATE.transform_keys();
                debug!("Transformed: {:?}", output);
                if INPUT_STATE.should_send_keyboard_event(&output) {
                    let backspace_count = INPUT_STATE.get_backspace_count();
                    debug!("Backspace count: {}", backspace_count);
                    _ = send_backspace(handle, backspace_count);
                    _ = send_string(handle, &output);
                    INPUT_STATE.replace(output);
                    return true;
                }
            }
        }
    }
    return false;
}

fn event_handler(handle: Handle, keycode: Option<char>, modifiers: KeyModifier) -> bool {
    unsafe {
        match keycode {
            Some(keycode) => {
                // Toggle Vietnamese input mod with Ctrl + Cmd + Space key
                if modifiers.is_control() && modifiers.is_super() && keycode == KEY_SPACE {
                    INPUT_STATE.toggle_vietnamese();
                    if let Some(event_sink) = UI_EVENT_SINK.get() {
                        _ = event_sink.submit_command(UPDATE_UI, (), Target::Auto);
                    }
                    return true;
                }

                if INPUT_STATE.is_enabled() {
                    match keycode {
                        KEY_ENTER | KEY_TAB | KEY_SPACE | KEY_ESCAPE => {
                            INPUT_STATE.new_word();
                        }
                        KEY_DELETE => {
                            INPUT_STATE.pop();
                        }
                        c => {
                            return process_character(handle, c, modifiers);
                        }
                    }
                }
            }
            None => {
                INPUT_STATE.new_word();
            }
        }
    }
    false
}

fn main() {
    env_logger::init();

    let win = WindowDesc::new(ui::main_ui_builder)
        .title("gõkey")
        .window_size((320.0, 200.0))
        .resizable(false);
    let app = AppLauncher::with_window(win);
    let event_sink = app.get_external_handle();
    _ = UI_EVENT_SINK.set(event_sink);

    thread::spawn(|| {
        run_event_listener(&event_handler);
    });

    _ = app.launch(UIDataAdapter::new());
}
