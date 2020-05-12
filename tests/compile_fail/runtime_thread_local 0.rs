pui::make_counter_tl! {
    type Foo = u8;
}

fn main() {
    fn assert_send_sync<T: Send + Sync>(_: T) {}
    assert_send_sync(Foo::new());
}
