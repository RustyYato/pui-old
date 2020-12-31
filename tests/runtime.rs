#[path = "runtime/macros.rs"]
mod macros;

#[test]
fn smoke() {
    pui::make_global_id_alloc! {
        #[derive(Debug)]
        type TinyIdAlloc(TinyId) = u8;
    }

    let a = TinyIdAlloc::new();
    let b = TinyIdAlloc::new();

    assert_ne!(a, b);

    assert_eq!(a.handle(), a.handle());
    assert_eq!(b.handle(), b.handle());

    assert_ne!(a.handle(), b.handle());
    assert_ne!(b.handle(), a.handle());
}

#[test]
fn exhaust_u8() {
    pui::make_global_id_alloc! {
        type TinyIdAlloc(TinyId) = u8;
    }

    let mut handles = Vec::new();

    for _ in 0..255 {
        handles.push(TinyIdAlloc::new().handle())
    }

    assert!(TinyIdAlloc::try_new().is_none());

    for (ai, a) in handles.iter().enumerate() {
        for (bi, b) in handles.iter().enumerate() {
            assert!((ai == bi) == (a == b));
        }
    }
}
