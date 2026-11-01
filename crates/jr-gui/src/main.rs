//! JoVee Ruler on its own: the harmonic field and the scale, no host.

fn main() -> Result<(), slint::PlatformError> {
    let ruler = jr_gui::Ruler::new()?;
    ruler.set_theme(jovee_theme::load(&jovee_theme::jovee_root()));
    ruler.set_key(0);
    slint::ComponentHandle::run(ruler.window())
}
