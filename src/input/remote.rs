//! Input handler para controle LG Magic Remote via SDL2.

use crate::error::{RusdiError, Result};
use crate::input::keycodes::{keycode_to_action, action_to_string, RusdiAction};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::sync::Arc;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum InputEvent {
    KeyPress(RusdiAction),
    KeyRelease(RusdiAction),
    KeyRepeat(RusdiAction),
    Quit,
}

pub struct InputHandler {
    event_tx: mpsc::UnboundedSender<InputEvent>,
    action_callback: Arc<dyn Fn(RusdiAction) + Send + Sync>,
}

impl InputHandler {
    pub fn new<F>(callback: F) -> (Self, mpsc::UnboundedReceiver<InputEvent>)
    where F: Fn(RusdiAction) + Send + Sync + 'static {
        let (tx, rx) = mpsc::unbounded_channel();
        (Self { event_tx: tx, action_callback: Arc::new(callback) }, rx)
    }

    pub fn run_sdl_loop(&self) -> Result<()> {
        let sdl = sdl2::init()?;
        let _video = sdl.video()?;
        let mut event_pump = sdl.event_pump()?;
        sdl.joystick()?;

        log_info!(crate::logging::init_logger(false), "SDL2 input loop started");

        loop {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => { let _ = self.event_tx.send(InputEvent::Quit); return Ok(()); }
                    Event::KeyDown { keycode: Some(kc), repeat, .. } => {
                        let action = self.sdl_keycode_to_action(kc);
                        let _ = self.event_tx.send(if repeat { InputEvent::KeyRepeat(action) } else { InputEvent::KeyPress(action) });
                    }
                    Event::KeyUp { keycode: Some(kc), .. } => {
                        let action = self.sdl_keycode_to_action(kc);
                        let _ = self.event_tx.send(InputEvent::KeyRelease(action));
                    }
                    Event::ControllerButtonDown { button, .. } => {
                        let _ = self.event_tx.send(InputEvent::KeyPress(self.sdl_gamepad_button_to_action(button)));
                    }
                    Event::ControllerButtonUp { button, .. } => {
                        let _ = self.event_tx.send(InputEvent::KeyRelease(self.sdl_gamepad_button_to_action(button)));
                    }
                    Event::ControllerAxisMotion { axis, value, .. } => {
                        if let Some(action) = self.sdl_gamepad_axis_to_action(axis, value) {
                            let _ = self.event_tx.send(InputEvent::KeyPress(action));
                        }
                    }
                    _ => {}
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    fn sdl_keycode_to_action(&self, kc: Keycode) -> RusdiAction {
        use Keycode::*; use RusdiAction::*;
        match kc {
            Up => Up, Down => Down, Left => Left, Right => Right,
            Return | KP_Enter => Select, Escape | Backspace => Back,
            Home => Home, Menu => ContextMenu, Space => PlayPause,
            P => Play, S => Stop, N => Next, R => Rewind, F => FastForward,
            PageUp => VolumeUp, PageDown => VolumeDown, M => Mute,
            F1 => Red, F2 => Green, F3 => Yellow, F4 => Blue,
            Num0 => Number(0), Num1 => Number(1), Num2 => Number(2), Num3 => Number(3),
            Num4 => Number(4), Num5 => Number(5), Num6 => Number(6), Num7 => Number(7),
            Num8 => Number(8), Num9 => Number(9),
            I => Info, E => Epg, T => Subtitle, A => Audio, Q => Exit,
            _ => Unknown(kc as u16),
        }
    }

    fn sdl_gamepad_button_to_action(&self, button: sdl2::controller::Button) -> RusdiAction {
        use sdl2::controller::Button::*; use RusdiAction::*;
        match button {
            DPadUp => Up, DPadDown => Down, DPadLeft => Left, DPadRight => Right,
            A => Select, B => Back, Guide => Home, Start => ContextMenu,
            LeftShoulder => Rewind, RightShoulder => FastForward,
            X => PlayPause, Y => Info, _ => Unknown(0),
        }
    }

    fn sdl_gamepad_axis_to_action(&self, axis: sdl2::controller::Axis, value: i16) -> Option<RusdiAction> {
        use sdl2::controller::Axis::*; use RusdiAction::*;
        const DZ: i16 = 8000;
        if value.abs() < DZ { return None; }
        match axis {
            LeftX if value < -DZ => Some(Left), LeftX if value > DZ => Some(Right),
            LeftY if value < -DZ => Some(Up), LeftY if value > DZ => Some(Down),
            _ => None,
        }
    }

    pub async fn process_events(&self, mut rx: mpsc::UnboundedReceiver<InputEvent>) {
        while let Some(event) = rx.recv().await {
            match event {
                InputEvent::KeyPress(a) | InputEvent::KeyRepeat(a) => {
                    (self.action_callback)(a);
                }
                InputEvent::Quit => break,
                _ => {}
            }
        }
    }
}
