//! JoVee Ruler on its own: the harmonic field and the scale, no host.

use std::time::Duration;

/// How often the shared config is read to see the look another JoVee app picked.
const LOOK_CHECK: Duration = Duration::from_secs(1);

fn main() -> Result<(), slint::PlatformError> {
    let ruler = std::rc::Rc::new(jr_gui::Ruler::new()?);
    let root = jovee_theme::jovee_root();
    ruler.set_theme(jovee_theme::load(&root));
    ruler.set_key(0);
    let timer = slint::Timer::default();
    let follows = ruler.clone();
    timer.start(slint::TimerMode::Repeated, LOOK_CHECK, move || {
        follows.set_theme(jovee_theme::load(&root));
    });
    slint::ComponentHandle::run(ruler.window())
}
