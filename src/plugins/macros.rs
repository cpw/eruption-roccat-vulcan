/*
    This file is part of Eruption.

    Eruption is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Eruption is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Eruption.  If not, see <http://www.gnu.org/licenses/>.
*/

use evdev_rs::enums::*;
use evdev_rs::{Device, InputEvent, TimeVal, UInputDevice};
use failure::Fail;
use lazy_static::lazy_static;
use log::*;
use parking_lot::Mutex;
use rlua::Context;
use std::any::Any;
use std::cell::RefCell;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;
use std::thread;

use crate::plugins::{self, Plugin};

pub type Result<T> = std::result::Result<T, MacrosPluginError>;

pub enum Message {
    MirrorKey(evdev_rs::InputEvent),
    InjectKey { key: u32, down: bool },
}

#[derive(Debug, Fail)]
pub enum MacrosPluginError {
    #[fail(display = "Could not open the evdev device")]
    EvdevError {},

    #[fail(display = "Could not spawn a thread")]
    ThreadSpawnError {},
    // #[fail(display = "Unknown error: {}", description)]
    // UnknownError { description: String },
}

lazy_static! {
    pub static ref UINPUT_TX: Arc<Mutex<Option<Sender<Message>>>> = Arc::new(Mutex::new(None));
    pub static ref DROP_CURRENT_KEY: AtomicBool = AtomicBool::new(false);
}

thread_local! {
    static DEVICE: RefCell<Option<UInputDevice>> = RefCell::new(None);
    static MODIFIER_PRESSED: RefCell<bool> = RefCell::new(false);
}

/// Implements support for macros by registering a virtual keyboard with the
/// system that mirrors keystrokes from the hardware keyboard
pub struct MacrosPlugin {}

impl MacrosPlugin {
    pub fn new() -> Self {
        MacrosPlugin {}
    }

    fn initialize_thread_locals() -> Result<()> {
        let dev = Device::new().unwrap();

        // setup virtual keyboard device
        dev.set_name("Eruption Virtual Keyboard");
        dev.set_bustype(3);
        dev.set_product_id(0x0123);
        dev.set_vendor_id(0x0059);
        dev.set_version(0x01);

        // configure allowed events
        dev.enable(&EventType::EV_KEY).unwrap();
        dev.enable(&EventType::EV_MSC).unwrap();
        dev.enable(&EventCode::EV_SYN(EV_SYN::SYN_REPORT)).unwrap();

        // enable media keys
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_PREVIOUSSONG))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_STOPCD)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_PLAYPAUSE))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_NEXTSONG))
            .unwrap();

        // Enable all supported keys; this is used to mirror the hardware device
        // to the virtual keyboard, so that the hardware device can be disabled.

        // Generated via `sudo evtest`
        // Input device name: "ROCCAT ROCCAT Vulcan AIMO"
        // Supported events:
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_ESC)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_1)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_2)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_3)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_4)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_5)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_6)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_7)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_8)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_9)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_0)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_MINUS)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_EQUAL)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_BACKSPACE))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_TAB)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_Q)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_W)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_E)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_R)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_T)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_Y)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_U)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_I)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_O)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_P)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_LEFTBRACE))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_RIGHTBRACE))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_ENTER)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_LEFTCTRL))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_A)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_S)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_D)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_F)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_G)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_H)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_J)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_K)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_L)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_SEMICOLON))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_APOSTROPHE))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_GRAVE)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_LEFTSHIFT))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_BACKSLASH))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_Z)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_X)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_C)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_V)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_B)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_N)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_M)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_COMMA)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_DOT)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_SLASH)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_RIGHTSHIFT))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_KPASTERISK))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_LEFTALT)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_SPACE)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_CAPSLOCK))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_F1)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_F2)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_F3)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_F4)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_F5)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_F6)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_F7)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_F8)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_F9)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_F10)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_NUMLOCK)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_SCROLLLOCK))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_KP7)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_KP8)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_KP9)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_KPMINUS)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_KP4)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_KP5)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_KP6)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_KPPLUS)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_KP1)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_KP2)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_KP3)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_KP0)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_KPDOT)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_ZENKAKUHANKAKU))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_102ND)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_F11)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_F12)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_RO)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_KATAKANA))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_HIRAGANA))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_HENKAN)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_KATAKANAHIRAGANA))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_MUHENKAN))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_KPJPCOMMA))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_KPENTER)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_RIGHTCTRL))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_KPSLASH)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_SYSRQ)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_RIGHTALT))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_HOME)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_UP)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_PAGEUP)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_LEFT)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_RIGHT)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_END)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_DOWN)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_PAGEDOWN))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_INSERT)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_DELETE)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_MUTE)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_VOLUMEDOWN))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_VOLUMEUP))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_POWER)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_KPEQUAL)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_PAUSE)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_KPCOMMA)).unwrap();
        //dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_HANGUEL)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_HANJA)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_YEN)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_LEFTMETA))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_RIGHTMETA))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_COMPOSE)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_STOP)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_AGAIN)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_PROPS)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_UNDO)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_FRONT)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_COPY)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_OPEN)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_PASTE)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_FIND)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_CUT)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_HELP)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_CALC)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_SLEEP)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_WWW)).unwrap();
        //dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_SCREENLOCK)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_BACK)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_FORWARD)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_EJECTCD)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_NEXTSONG))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_PLAYPAUSE))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_PREVIOUSSONG))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_STOPCD)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_REFRESH)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_EDIT)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_SCROLLUP))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_SCROLLDOWN))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_KPLEFTPAREN))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_KPRIGHTPAREN))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_F13)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_F14)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_F15)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_F16)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_F17)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_F18)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_F19)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_F20)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_F21)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_F22)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_F23)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_F24)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_UNKNOWN)).unwrap();

        match UInputDevice::create_from_device(&dev) {
            Ok(device) => {
                DEVICE.with(|dev| *dev.borrow_mut() = Some(device));

                Ok(())
            }

            Err(_e) => Err(MacrosPluginError::EvdevError {}),
        }
    }

    /// Inject a press or release of key `key` into to output of the virtual keyboard
    fn inject_single_key(key: EV_KEY, value: i32, time: &TimeVal) -> Result<()> {
        //let mut do_initialize = false;

        DEVICE.with(|dev| {
            let device = dev.borrow();

            if let Some(device) = device.as_ref() {
                let event = InputEvent {
                    time: time.clone(),
                    event_type: EventType::EV_KEY,
                    event_code: EventCode::EV_KEY(key.clone()),
                    value,
                };

                device.write_event(&event).unwrap();

                let event = InputEvent {
                    time: time.clone(),
                    event_type: EventType::EV_SYN,
                    event_code: EventCode::EV_SYN(EV_SYN::SYN_REPORT),
                    value,
                };

                device.write_event(&event).unwrap();
            } else {
                error!("Inconsistent thread local storage state detected");
                //do_initialize = true;
            }
        });

        //if do_initialize {
            //Self::initialize_thread_locals().unwrap();
        //}

        Ok(())
    }

    /// Inject a pre-existing InputEvent into to output of the virtual keyboard
    fn inject_key_event(event: evdev_rs::InputEvent) -> Result<()> {
        let mut do_initialize = false;

        DEVICE.with(|dev| {
            debug!("Injecting: {:?}", event);

            if let Some(device) = dev.borrow().as_ref() {
                device.write_event(&event).unwrap();
            } else {
                do_initialize = true;
            }
        });

        if do_initialize {
            Self::initialize_thread_locals().unwrap();
        }

        Ok(())
    }

    fn spawn_uinput_thread() -> Result<()> {
        let (uinput_tx, uinput_rx) = channel();

        thread::Builder::new()
            .name("uinput".into())
            .spawn(move || {
                Self::initialize_thread_locals().unwrap();

                // media keys will be handled by the Lua script 'macros.lua'

                // register an event observer for processing of all raw
                // keyboard events from the hardware keyboard
                //events::register_observer(|event: &events::Event| {
                    //match event {
                        //events::Event::RawKeyboardEvent(raw_event) => {
                            ////debug!("Key event: {:?}", raw_event);

                            //let mut already_processed = false;

                            //if let EventCode::EV_KEY(ref code) = raw_event.event_code {
                                //if code == &EV_KEY::KEY_RIGHTCTRL {
                                    //MODIFIER_PRESSED.with(|modifier| {
                                        //*modifier.borrow_mut() = raw_event.value > 0
                                    //});
                                //}

                                //// support media keys
                                //MODIFIER_PRESSED.with(|modifier| {
                                    //if raw_event.value < 2 {
                                        //trace!("Modifier pressed: {}", *modifier.borrow());

                                        //if code == &EV_KEY::KEY_F9 && *modifier.borrow() {
                                            //Self::inject_single_key(
                                                //EV_KEY::KEY_PREVIOUSSONG,
                                                //raw_event.value,
                                                //&raw_event.time,
                                            //)
                                            //.unwrap();
                                            //already_processed = true;
                                        //}

                                        //if code == &EV_KEY::KEY_F10 && *modifier.borrow() {
                                            //Self::inject_single_key(
                                                //EV_KEY::KEY_STOPCD,
                                                //raw_event.value,
                                                //&raw_event.time,
                                            //)
                                            //.unwrap();
                                            //already_processed = true;
                                        //}

                                        //if code == &EV_KEY::KEY_F11 && *modifier.borrow() {
                                            //Self::inject_single_key(
                                                //EV_KEY::KEY_PLAYPAUSE,
                                                //raw_event.value,
                                                //&raw_event.time,
                                            //)
                                            //.unwrap();
                                            //already_processed = true;
                                        //}

                                        //if code == &EV_KEY::KEY_F12 && *modifier.borrow() {
                                            //Self::inject_single_key(
                                                //EV_KEY::KEY_NEXTSONG,
                                                //raw_event.value,
                                                //&raw_event.time,
                                            //)
                                            //.unwrap();
                                            //already_processed = true;
                                        //}
                                    //}
                                //});
                            //}

                            //if !already_processed [>&& !DROP_CURRENT_KEY.load(Ordering::SeqCst)<] {
                                //// mirror hardware keyboard to virtual keyboard
                                //Self::inject_key_event(raw_event.clone()).unwrap();
                            //}

                            //Ok(true)
                        //}

                        //_ => Ok(false),
                    //}
                //});

                loop {
                    let message = uinput_rx.recv().unwrap();
                    match message {
                        Message::MirrorKey(raw_event) => {
                            if !DROP_CURRENT_KEY.load(Ordering::SeqCst) {
                                Self::inject_key_event(raw_event).unwrap();
                            } else {
                                debug!("Original input has been dropped, as requested");
                            }
                        }

                        Message::InjectKey { key: ev_key, down } => {
                            let key = evdev_rs::enums::int_to_ev_key(ev_key).unwrap_or_else(|| {
                                error!("Invalid key index");
                                panic!()
                            });

                            let value = if down { 1 } else { 0 };

                            let mut time: libc::timeval = libc::timeval {
                                tv_sec: 0,
                                tv_usec: 0,
                            };

                            unsafe {
                                libc::gettimeofday(&mut time, std::ptr::null_mut());
                            }

                            let time = evdev_rs::TimeVal::from_raw(&time);

                            Self::inject_single_key(key, value, &time).unwrap();
                        }
                    }
                }
            })
            .map_err(|_e| MacrosPluginError::ThreadSpawnError {})?;

        *UINPUT_TX.lock() = Some(uinput_tx);

        Ok(())
    }
}

impl Plugin for MacrosPlugin {
    fn get_name(&self) -> String {
        "Macros".to_string()
    }

    fn get_description(&self) -> String {
        "Inject programmable keyboard events".to_string()
    }

    fn initialize(&mut self) -> plugins::Result<()> {
        Self::spawn_uinput_thread()?;

        Ok(())
    }

    fn register_lua_funcs(&self, _lua_ctx: Context) -> rlua::Result<()> {
        Ok(())
    }

    fn main_loop_hook(&self, _ticks: u64) {}

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
