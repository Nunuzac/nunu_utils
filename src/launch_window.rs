use navigation_utils::kitty;
use std::env;

fn main() {
    let location = env::args().nth(1);
    if let Some(mut instance) = kitty::new_kitty()
        && let Some(tab) = instance.get_focused_tab()
    {
        let cwd = tab.get_focused_window().map(|window| window.cwd);
        instance.launch_window(&tab.id, cwd, location);
    }
}
