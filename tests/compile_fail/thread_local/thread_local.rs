pui::make_typeid_tl! {
    type Id;
}

fn main() {
    fn assert_send_sync<T: Send + Sync>(t: T) {}
    assert_send_sync(Id::new());
}
