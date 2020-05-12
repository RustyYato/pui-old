#[path = "runtime/macros.rs"]
mod macros;

#[test]
fn smoke() {
    pui::make_counter! {
        #[derive(Debug)]
        type TinyCounter = u8;
    }

    let a = TinyCounter::new();
    let b = TinyCounter::new();

    assert_ne!(a, b);

    assert_eq!(a.handle(), a.handle());
    assert_eq!(b.handle(), b.handle());

    assert_ne!(a.handle(), b.handle());
    assert_ne!(b.handle(), a.handle());
}

#[test]
fn exhaust_u8() {
    pui::make_counter! {
        type TinyCounter = u8;
    }

    let mut handles = Vec::new();

    for _ in 0..255 {
        handles.push(TinyCounter::new().handle())
    }

    assert!(TinyCounter::try_new().is_none());

    for (ai, a) in handles.iter().enumerate() {
        for (bi, b) in handles.iter().enumerate() {
            assert!((ai == bi) == (a == b));
        }
    }
}
