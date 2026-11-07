#[macro_use]
extern crate log;
use navigation_utils::common::{Direction, Focusable};
use navigation_utils::focus::new_instance;
use simplelog::{Config, LevelFilter, WriteLogger};
use std::env;
use std::fs::File;

fn main() {
    if env::var("NAVIGATOR_LOGGING_ENABLE").is_ok() {
        WriteLogger::init(
            LevelFilter::Debug,
            Config::default(),
            File::create("/tmp/navigator.log").unwrap(),
        )
        .unwrap();
    }
    let direction = env::args().nth(1);
    if let Some(direction) = &direction
        && let Ok(direction) = Direction::try_from(direction.as_ref())
    {
        info!("attempting to move to {}", direction);
        let _ = new_instance().move_focus(&direction);
    } else {
        info!("cannot move to direction: {:?}", direction);
    }
    info!("session finished")
}
