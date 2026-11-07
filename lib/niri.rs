use crate::common::{Direction, Focusable};
use niri_ipc::{Action, Request, socket::Socket};

pub struct Niri;

impl Focusable for Niri {
    fn move_focus(&self, direction: &Direction) -> Result<(), String> {
        if let Ok(mut socket) = Socket::connect() {
            let action = match direction {
                Direction::LEFT => Action::FocusColumnLeft {},
                Direction::RIGHT => Action::FocusColumnRight {},
                Direction::UP => Action::FocusWindowUp {},
                Direction::DOWN => Action::FocusWindowDown {},
            };
            let _ = socket.send(Request::Action(action));
        }
        Ok(())
    }

    fn delete_focus(&mut self) -> Result<(), String> {
        if let Ok(mut socket) = Socket::connect() {
            let _ = socket.send(Request::Action(Action::CloseWindow { id: None }));
        }
        Ok(())
    }

    fn resize_focus(&mut self, _: &Direction, _: &u8) -> Result<(), String> {
        Ok(())
    }
}
