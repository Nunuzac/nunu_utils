use navigation_utils::{
    common::{Direction, Focusable},
    focus::new_instance,
};
use std::env;

fn main() {
    let direction = env::args()
        .nth(1)
        .and_then(|it| Direction::try_from(it.as_ref()).ok());
    let amount = env::args()
        .nth(2)
        .and_then(|it| it.parse::<u8>().ok())
        .unwrap_or(2);
    if let Some(direction) = direction {
        let _ = new_instance().resize_focus(&direction, &amount);
    }
}
