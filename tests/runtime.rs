#[path = "runtime/macros.rs"]
mod macros;

#[test]
fn smoke() {
    pui::make_counter! {
        #[derive(Debug)]
        type TinyCounter = u8;
    }

    let a = TinyCounter::new_runtime();
    let b = TinyCounter::new_runtime();

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
        handles.push(TinyCounter::new_runtime().handle())
    }

    assert!(TinyCounter::try_new_runtime().is_none());

    for (ai, a) in handles.iter().enumerate() {
        for (bi, b) in handles.iter().enumerate() {
            assert!((ai == bi) == (a == b));
        }
    }
}
