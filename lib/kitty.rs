use log::*;
use procfs::{ProcError, net::unix};
use serde::Deserialize;
use std::{io::Error, os::unix::process::CommandExt, process::Command};

use crate::common::{Direction, Focusable};

#[derive(Clone)]
pub struct Kitty {
    socket_addr: String,
    focused_os_window: Option<OsWindow>,
    focused_tab: Option<Tab>,
    focused_window: Option<Window>,
}

#[derive(Clone, Deserialize, Debug)]
pub struct WindowNeighbors {
    pub left: Option<Vec<u16>>,
    pub right: Option<Vec<u16>>,
    pub top: Option<Vec<u16>>,
    pub bottom: Option<Vec<u16>>,
}

#[derive(Clone, Deserialize, Debug)]
pub struct ForegroundProcess {
    pub pid: u32,
    pub cmdline: Vec<String>,
    pub cwd: String,
}

#[derive(Clone, Deserialize, Debug)]
pub struct Window {
    pub cwd: String,
    pub id: u16,
    pub is_active: bool,
    pub is_focused: bool,
    pub is_self: bool,
    pub neighbors: WindowNeighbors,
    pub pid: u32,
    pub cmdline: Vec<String>,
    pub foreground_processes: Vec<ForegroundProcess>,
}

#[derive(Clone, Deserialize, Debug)]
pub struct Tab {
    pub id: u16,
    pub is_active: bool,
    pub is_focused: bool,
    pub layout: String,
    pub title: String,
    pub windows: Vec<Window>,
}

#[derive(Clone, Deserialize, Debug)]
pub struct OsWindow {
    pub id: u16,
    pub is_active: bool,
    pub is_focused: bool,
    pub tabs: Vec<Tab>,
}

fn find_kitty_socket() -> Result<String, ProcError> {
    match unix() {
        Ok(sockets) => sockets
            .iter()
            .filter_map(|it| it.path.to_owned())
            .filter_map(|it| it.into_os_string().into_string().ok())
            .find(|it| it.contains("kitty-"))
            .ok_or(ProcError::from("socket not listed in pseudofs"))
            .inspect_err(|it| info!("no kitty socket found: {}", it)),

        Err(err) => Err(err),
    }
}

pub fn new_kitty() -> Option<Kitty> {
    if let Ok(socket) = find_kitty_socket() {
        info!("kitty socket found at: {}", socket);
        Some(Kitty {
            socket_addr: format!("unix:{}", socket),
            focused_os_window: None,
            focused_tab: None,
            focused_window: None,
        })
    } else {
        None
    }
}

pub fn launch_kitty(cwd: Option<String>) -> Error {
    let mut command = Command::new("kitty");
    if let Some(launch_cwd) = cwd {
        command.arg(format!("--working-dir={}", launch_cwd));
    }
    command.exec()
}

fn kitty_direction(direction: &Direction) -> String {
    match direction {
        Direction::LEFT => "left",
        Direction::RIGHT => "right",
        Direction::UP => "top",
        Direction::DOWN => "bottom",
    }
    .to_string()
}

impl Kitty {
    fn at(&self) -> Command {
        let mut cmd = Command::new("kitty");
        cmd.arg("@").arg(format!("--to={}", self.socket_addr));
        cmd
    }

    pub fn launch_window(&self, id: &u16, cwd: Option<String>, location: Option<String>) -> Error {
        let mut cmd = self.at();
        cmd.arg("launch").arg(format!("--match=id:{}", id));
        if let Some(loc) = location {
            cmd.arg(format!("--location={}", loc));
        }
        if let Some(path) = cwd {
            cmd.arg(format!("--cwd={}", path));
        }
        cmd.exec()
    }

    fn ls(&self, mtch: Option<&str>) -> Result<Vec<OsWindow>, Error> {
        let mut cmd = self.at();
        cmd.arg("ls");
        if let Some(m) = mtch {
            cmd.arg(format!("--match={}", m));
        }
        let output = cmd.output()?;
        serde_json::from_slice(&output.stdout).map_err(Error::from)
    }

    pub fn launch_os_window(&self, cwd: Option<String>) -> Error {
        let mut cmd = self.at();
        cmd.arg("launch").arg("--type=os-window");
        if let Some(launch_cwd) = cwd {
            cmd.arg(format!("--cwd={}", launch_cwd));
        }
        cmd.exec()
    }

    pub fn get_focused_os_window(&mut self) -> Option<OsWindow> {
        if self.focused_os_window.is_none() {
            self.focused_os_window = self
                .ls(Some("state:focused"))
                .inspect_err(|err| warn!("no focused kitty window found: {}", err))
                .ok()
                .and_then(|it| it.first().cloned())
                .filter(|os_window| os_window.is_focused)
                .inspect(|os_window| info!("kitty active os_window id: {}", os_window.id));
        }
        self.focused_os_window.clone()
    }

    pub fn get_focused_tab(&mut self) -> Option<Tab> {
        if self.focused_tab.is_none() {
            self.focused_tab = self
                .get_focused_os_window()
                .as_ref()
                .and_then(|os_window| os_window.get_focused_tab())
                .filter(|tab| tab.is_focused);
        }
        self.focused_tab.clone()
    }

    pub fn get_focused_window(&mut self) -> Option<Window> {
        if self.focused_window.is_none() {
            self.focused_window = self
                .get_focused_tab()
                .as_ref()
                .and_then(|tab| tab.get_focused_window())
                .filter(|window| window.is_focused);
        }
        self.focused_window.clone()
    }

    pub fn focus_window(&self, mtch: String) -> bool {
        self.at()
            .arg("focus-window")
            .arg(format!("--match={}", mtch))
            .output()
            .inspect_err(|err| warn!("unable to move kitty focus to {}: {}", mtch, err))
            .is_ok_and(|exit_status| exit_status.status.success())
    }

    pub fn close_window(&self, wid: &u16) -> Result<(), String> {
        self.at()
            .arg("close-window")
            .arg(format!("--match=id:{}", wid))
            .output()
            .inspect_err(|err| error!("unable to close kitty window {}: {}", wid, err))
            .map_err(|err| err.to_string())
            .map(|_| ())
    }

    fn can_move(&self, direction: &Direction) -> bool {
        let dir = kitty_direction(direction);
        info!("attempting to move kitty to: {}", dir);
        self.ls(Some(&format!("neighbor:{}", dir)))
            .inspect_err(|err| warn!("unable to move kitty to {}: {}", dir, err))
            .ok()
            .and_then(|it| it.first().cloned())
            .inspect(|os_window| {
                info!(
                    "found os_window at position {} with id {}, is focused {}",
                    dir, os_window.id, os_window.is_focused
                )
            })
            .is_some_and(|os_window| os_window.is_focused)
    }

    fn resize_window(&mut self, axis: &str, amout: &i8) -> Result<(), String> {
        if let Some(window) = self.get_focused_window() {
            self.at()
                .arg("resize-window")
                .arg(format!("--match=id:{}", window.id))
                .arg(format!("--increment={}", amout))
                .arg(format!("--axis={}", axis))
                .output()
                .inspect_err(|err| error!("unable to resize kitty window {}: {}", window.id, err))
                .map_err(|err| err.to_string())
                .map(|_| ())
        } else {
            Ok(())
        }
    }
}

impl Focusable for Kitty {
    fn move_focus(&self, direction: &Direction) -> Result<(), String> {
        let dir = kitty_direction(direction);
        if self.can_move(direction) && self.focus_window(format!("neighbor:{}", dir)) {
            info!("kitty focus moved to {}", dir);
            Ok(())
        } else {
            warn!("kitty focus could not be moved to {}", direction);
            Err(String::from("no window to move to"))
        }
    }

    fn delete_focus(&mut self) -> Result<(), String> {
        self.get_focused_window()
            .ok_or_else(|| String::from("window not found"))
            .and_then(|it| self.close_window(&it.id))
    }

    fn resize_focus(&mut self, direction: &Direction, amount: &u8) -> Result<(), String> {
        let (axis, amt) = match direction {
            Direction::LEFT => ("horizontal", *amount as i8),
            Direction::RIGHT => ("horizontal", -(*amount as i8)),
            Direction::UP => ("vertical", *amount as i8),
            Direction::DOWN => ("vertical", -(*amount as i8)),
        };
        self.resize_window(axis, &amt)
    }
}

impl OsWindow {
    pub fn get_focused_tab(&self) -> Option<Tab> {
        self.tabs
            .iter()
            .find(|tab| tab.is_active)
            .inspect(|tab| info!("kitty active tab found id: {}", tab.id))
            .cloned()
    }
}

impl Tab {
    pub fn get_focused_window(&self) -> Option<Window> {
        self.windows
            .iter()
            .find(|window| window.is_active)
            .inspect(|window| info!("kitty active window found id: {}", window.id))
            .cloned()
    }
}

impl Window {
    pub fn neighbor(&self, direction: &Direction) -> Option<Vec<u16>> {
        match direction {
            Direction::LEFT => self.neighbors.left.as_ref().cloned(),
            Direction::RIGHT => self.neighbors.right.as_ref().cloned(),
            Direction::UP => self.neighbors.top.as_ref().cloned(),
            Direction::DOWN => self.neighbors.bottom.as_ref().cloned(),
        }
    }
}
