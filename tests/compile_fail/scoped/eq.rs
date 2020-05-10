use pui::scoped::Scoped;

fn main() {
    Scoped::with(|a| Scoped::with(|b| assert_eq!(a, b)))
}
