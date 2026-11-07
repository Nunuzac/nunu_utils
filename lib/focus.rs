use log::info;

use crate::{
    common::{Direction, Focusable},
    kitty::{self, Kitty, Window},
    niri::Niri,
    nvim::{self, Nvim},
};

pub struct FocusHandler {
    niri: Niri,
    nvim: Option<Nvim>,
    kitty: Option<Kitty>,
}

fn get_nvim_pid(window: &Window) -> Option<i32> {
    window
        .foreground_processes
        .iter()
        .find(|it| it.cmdline.iter().any(|it| it == "nvim"))
        .map(|proc| proc.pid as i32)
        .inspect(|pid| {
            info!(
                "found nvim with pid {} in window with id {}",
                pid, window.id
            )
        })
}

pub fn new_instance() -> FocusHandler {
    let niri = Niri {};
    let mut kitty = kitty::new_kitty();
    let mut nvim = None;
    if let Some(kitty) = &mut kitty
        && let Some(window) = kitty.get_focused_window()
        && let Some(pid) = get_nvim_pid(&window)
    {
        nvim = nvim::new_nvim(pid).ok()
    }
    FocusHandler { niri, kitty, nvim }
}

impl Focusable for FocusHandler {
    fn move_focus(&self, direction: &Direction) -> Result<(), String> {
        let nvim_moved = self
            .nvim
            .as_ref()
            .is_some_and(|v| v.move_focus(direction).is_ok());
        info!("nvim focus moved: {}", nvim_moved);
        if nvim_moved
            || self
                .kitty
                .as_ref()
                .is_some_and(|k| k.move_focus(direction).is_ok())
        {
            Ok(())
        } else {
            info!("moving niri to {}", direction);
            self.niri.move_focus(direction)
        }
    }

    fn delete_focus(&mut self) -> Result<(), String> {
        if let Some(nv) = &mut self.nvim {
            nv.delete_focus()
        } else if let Some(kt) = &mut self.kitty
            && kt.get_focused_window().is_some()
        {
            kt.delete_focus()
        } else {
            self.niri.delete_focus()
        }
    }

    fn resize_focus(&mut self, direction: &Direction, amount: &u8) -> Result<(), String> {
        if let Some(nv) = &mut self.nvim {
            nv.resize_focus(direction, amount)
        } else if let Some(kt) = &mut self.kitty
            && kt.get_focused_window().is_some()
        {
            kt.resize_focus(direction, amount)
        } else {
            Ok(())
        }
    }
}
