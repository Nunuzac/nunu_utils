use crate::common::{Direction, Focusable, get_children};
use log::*;
use std::{
    ffi::OsString,
    io::Error,
    process::{Command, Output},
};

pub struct Nvim {
    server_addr: String,
}

pub fn new_nvim(pid: i32) -> Result<Nvim, Error> {
    let socket_addr = get_children(pid)
        .flat_map(|proc| get_children(proc.pid))
        .filter_map(|proc| proc.environ().ok())
        .find(|env| env.contains_key(&OsString::from("NVIM")))
        .and_then(|env| env.get(&OsString::from("NVIM")).cloned())
        .and_then(|it| it.into_string().ok())
        .inspect(|socket_addr| info!("found nvim socket with address {}", socket_addr));
    if let Some(socket_addr) = socket_addr {
        return Ok(Nvim {
            server_addr: socket_addr,
        });
    }
    warn!(
        "nvim socket address not found but nvim is running with pid {}",
        pid
    );
    Err(Error::last_os_error())
}

fn get_id_from_out(output: Output) -> Option<u16> {
    String::from_utf8(output.stdout)
        .ok()
        .and_then(|it| it.parse::<u16>().ok())
}

fn get_key_mapping(dir: &Direction) -> String {
    match dir {
        Direction::LEFT => "h",
        Direction::RIGHT => "l",
        Direction::UP => "k",
        Direction::DOWN => "j",
    }
    .into()
}

impl Nvim {
    fn run_in_server(&self) -> Command {
        let mut cmd = Command::new("nvim");
        cmd.arg("--headless")
            .arg("--clean")
            .arg("--server")
            .arg(self.server_addr.clone());
        cmd
    }

    pub fn get_window_id(&self, args: Option<String>) -> Option<u16> {
        let mut cmd = self.run_in_server();
        cmd.arg("--remote-expr");
        if let Some(a) = &args {
            cmd.arg(format!("win_getid(winnr(\"{}\"))", a));
        } else {
            cmd.arg("win_getid(winnr())");
        }

        cmd.output()
            .inspect_err(|err| {
                error!(
                    "unable to get nvim window id with opts {:?}: {}",
                    &args, err
                )
            })
            .ok()
            .and_then(get_id_from_out)
    }

    fn can_move(&self, direction: &crate::common::Direction) -> Option<u16> {
        let curr = self.get_window_id(None);
        let other = self.get_window_id(Some(get_key_mapping(direction)));
        if other.is_some() && curr != other {
            info!("nvim can move to: {}", direction);
            other
        } else {
            info!("nvim cannot move to: {}", direction);
            None
        }
    }

    fn move_focus_by_id(&self, id: u16) -> bool {
        self.run_in_server()
            .arg("--remote-expr")
            .arg(format!("win_gotoid({})", id))
            .output()
            .inspect_err(|err| error!("unable to move nvim window: {}", err))
            .is_ok_and(|out| out.status.success())
    }

    fn quit_all(&self) -> Result<(), String> {
        self.run_in_server()
            .arg("--remote-expr")
            .arg("execute(\"qa\")")
            .output()
            .inspect_err(|err| error!("unable to close nvim: {}", err))
            .map_err(|err| err.to_string())
            .map(|_| ())
    }

    fn resize_window(&self, orientation: &str, amt: &str) -> Result<(), String> {
        self.run_in_server()
            .arg("--remote-send")
            .arg(format!("<C-\\><C-N>:{} resize {}<CR>", orientation, amt))
            .output()
            .inspect_err(|err| error!("unable to resize nvim window: {}", err))
            .map_err(|err| err.to_string())
            .map(|_| ())
    }
}

impl Focusable for Nvim {
    fn move_focus(&self, direction: &Direction) -> Result<(), String> {
        if let Some(wid) = self.can_move(direction)
            && self.move_focus_by_id(wid)
        {
            Ok(())
        } else {
            Err(String::from("unable to move nvim"))
        }
    }

    fn delete_focus(&mut self) -> Result<(), String> {
        self.quit_all()
    }

    fn resize_focus(&mut self, direction: &Direction, amount: &u8) -> Result<(), String> {
        let (orientation, amt) = match direction {
            Direction::LEFT => ("vertical", format!("+{}", amount)),
            Direction::RIGHT => ("vertical", format!("-{}", amount)),
            Direction::UP => ("", format!("-{}", amount)),
            Direction::DOWN => ("", format!("+{}", amount)),
        };
        println!("{} @ {}", orientation, amt);
        self.resize_window(orientation, &amt)
    }
}
