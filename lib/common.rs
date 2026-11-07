use procfs::process::Process;
use std::fmt::Display;

#[derive(Debug)]
pub enum Direction {
    LEFT,
    RIGHT,
    UP,
    DOWN,
}

impl Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::LEFT => write!(f, "left"),
            Direction::RIGHT => write!(f, "right"),
            Direction::UP => write!(f, "up"),
            Direction::DOWN => write!(f, "down"),
        }
    }
}

impl TryFrom<&str> for Direction {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "left" => Ok(Direction::LEFT),
            "right" => Ok(Direction::RIGHT),
            "up" => Ok(Direction::UP),
            "down" => Ok(Direction::DOWN),
            _ => Err(format!("unknown direction '{}'", value)),
        }
    }
}

pub trait Focusable {
    fn move_focus(&self, direction: &Direction) -> Result<(), String>;
    fn delete_focus(&mut self) -> Result<(), String>;
    fn resize_focus(&mut self, direction: &Direction, amount: &u8) -> Result<(), String>;
}

pub fn get_children(pid: i32) -> impl Iterator<Item = Process> {
    let proc = Process::new(pid).ok();
    proc.into_iter()
        .filter_map(|proc| proc.tasks().ok())
        .flatten()
        .filter_map(|task| task.ok())
        .filter_map(|task| task.children().ok())
        .flatten()
        .filter_map(|pid| Process::new(pid as i32).ok())
}
