use pui::typeid::{Type, TypeHandle};

#[test]
fn smoke() {
    pui::make_typeid! {
        type MyTypeid;
    }

    const _: [(); 0] = [(); std::mem::size_of::<MyTypeid>()];
    const _: [(); 0] = [(); std::mem::size_of::<Type<MyTypeid>>()];
    const _: [(); 0] = [(); std::mem::size_of::<TypeHandle<MyTypeid>>()];

    let _x = MyTypeid::new();
    assert!(MyTypeid::try_new().is_none());
}

#[test]
#[cfg(feature = "std")]
fn thread_local() {
    use pui::test_setup::ThreadGroup;

    pui::make_typeid_tl! {
        type MyTypeId;
    }

    const _: [(); 0] = [(); std::mem::size_of::<MyTypeId>()];
    const _: [(); 0] = [(); std::mem::size_of::<Type<MyTypeId>>()];
    const _: [(); 0] = [(); std::mem::size_of::<TypeHandle<MyTypeId>>()];

    let thread_group = ThreadGroup::new(2);

    thread_group.spawn(move |_, wait| {
        let _a = MyTypeId::new();
        wait.wait();
    });

    thread_group.spawn(move |_, wait| {
        let _a = MyTypeId::new();
        wait.wait();
    });
}

#[test]
#[cfg(feature = "std")]
fn multi_threaded() {
    use pui::test_setup::ThreadGroup;

    pui::make_typeid! {
        type MyTypeId;
    }

    const _: [(); 0] = [(); std::mem::size_of::<MyTypeId>()];
    const _: [(); 0] = [(); std::mem::size_of::<Type<MyTypeId>>()];
    const _: [(); 0] = [(); std::mem::size_of::<TypeHandle<MyTypeId>>()];

    let thread_group = ThreadGroup::new(2);

    thread_group.spawn(move |_, wait| {
        wait.wait();
        assert!(MyTypeId::try_new().is_none());
        wait.wait();
    });

    thread_group.spawn(move |_, wait| {
        let _a = MyTypeId::new();
        wait.wait();
        wait.wait();
    });
}
