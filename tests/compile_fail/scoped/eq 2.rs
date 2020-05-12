fn main() {
    pui::make_scoped!(a);
    pui::make_scoped!(b);
    assert_eq!(a, b);
}
