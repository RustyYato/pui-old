use pui::runtime::Global;

pui::make_global_pool! {
    thread_local one Foo(Global);
}

fn main() {
    fn assert_send_sync<T: Send + Sync>(_: T) {}
    assert_send_sync(Global::new());
    assert_send_sync(Global::with_pool(Foo))
}
