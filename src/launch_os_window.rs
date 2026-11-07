use navigation_utils::kitty;
use std::env;

fn main() {
    let cwd = env::args().nth(1);
    if let Some(mut instance) = kitty::new_kitty() {
        if cwd.is_none()
            && let Some(window) = instance.get_focused_window()
        {
            instance.launch_os_window(Some(window.cwd));
        } else {
            instance.launch_os_window(cwd);
        }
    } else {
        kitty::launch_kitty(cwd);
    }
}
