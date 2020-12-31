use pui::{
    make_global_pool,
    runtime::{Global, GlobalId},
};

#[cfg(feature = "std")]
make_global_pool! {
    stack Stack(GlobalId);
}

#[cfg(feature = "std")]
make_global_pool! {
    thread_local stack StackTl(GlobalId);
}

#[cfg(feature = "std")]
make_global_pool! {
    queue Queue(GlobalId);
}

#[cfg(feature = "std")]
make_global_pool! {
    thread_local queue QueueTl(GlobalId);
}

make_global_pool! {
    one One(GlobalId);
}

#[cfg(feature = "std")]
make_global_pool! {
    thread_local one OneTl(GlobalId);
}

#[test]
fn no_pool() {
    let (a, b);
    {
        let rt_a = Global::new();
        let rt_b = Global::new();
        a = rt_a.handle();
        b = rt_b.handle();
        assert_ne!(rt_a.handle(), rt_b.handle());
    }

    {
        let rt_a = Global::new();
        let rt_b = Global::new();
        assert_ne!(rt_a.handle(), rt_b.handle());
        assert_ne!(rt_a.handle(), a);
        assert_ne!(rt_b.handle(), a);
        assert_ne!(rt_a.handle(), b);
        assert_ne!(rt_b.handle(), b);
    }
}

#[test]
#[cfg(feature = "std")]
fn stack() {
    let (a, b);
    {
        let rt_a = Global::with_pool(Stack);
        let rt_b = Global::with_pool(Stack);
        a = rt_a.handle();
        b = rt_b.handle();
        assert_ne!(rt_a.handle(), rt_b.handle());
    }

    {
        let rt_a = Global::with_pool(Stack);
        let rt_b = Global::with_pool(Stack);
        assert_ne!(rt_a.handle(), rt_b.handle());
        assert_eq!(rt_a.handle(), a);
        assert_eq!(rt_b.handle(), b);
    }
}

#[test]
#[cfg(feature = "std")]
fn stack_tl() {
    let (a, b);
    {
        let rt_a = Global::with_pool(StackTl);
        let rt_b = Global::with_pool(StackTl);
        a = rt_a.handle();
        b = rt_b.handle();
        assert_ne!(rt_a.handle(), rt_b.handle());
    }

    {
        let rt_a = Global::with_pool(StackTl);
        let rt_b = Global::with_pool(StackTl);
        assert_ne!(rt_a.handle(), rt_b.handle());
        assert_eq!(rt_a.handle(), a);
        assert_eq!(rt_b.handle(), b);
    }
}

#[test]
#[cfg(feature = "std")]
fn queue() {
    let (a, b);
    {
        let rt_a = Global::with_pool(Queue);
        let rt_b = Global::with_pool(Queue);
        a = rt_a.handle();
        b = rt_b.handle();
        assert_ne!(rt_a.handle(), rt_b.handle());
        drop(rt_a);
        drop(rt_b);
    }

    {
        let rt_a = Global::with_pool(Queue);
        let rt_b = Global::with_pool(Queue);
        assert_ne!(rt_a.handle(), rt_b.handle());
        assert_eq!(rt_a.handle(), a);
        assert_eq!(rt_b.handle(), b);
    }
}

#[test]
#[cfg(feature = "std")]
fn queue_tl() {
    let (a, b);
    {
        let rt_a = Global::with_pool(QueueTl);
        let rt_b = Global::with_pool(QueueTl);
        a = rt_a.handle();
        b = rt_b.handle();
        assert_ne!(rt_a.handle(), rt_b.handle());
        drop(rt_a);
        drop(rt_b);
    }

    {
        let rt_a = Global::with_pool(QueueTl);
        let rt_b = Global::with_pool(QueueTl);
        assert_ne!(rt_a.handle(), rt_b.handle());
        assert_eq!(rt_a.handle(), a);
        assert_eq!(rt_b.handle(), b);
    }
}

#[test]
fn one() {
    let (a, b);
    {
        let rt_a = Global::with_pool(One);
        let rt_b = Global::with_pool(One);
        a = rt_a.handle();
        b = rt_b.handle();
        assert_ne!(rt_a.handle(), rt_b.handle());
    }

    {
        let rt_a = Global::with_pool(One);
        let rt_b = Global::with_pool(One);
        assert_ne!(rt_a.handle(), rt_b.handle());
        assert_eq!(rt_a.handle(), b);
        assert_ne!(rt_b.handle(), a);
        assert_ne!(rt_b.handle(), b);
    }
}

#[test]
#[cfg(feature = "std")]
fn one_tl() {
    let (a, b);
    {
        let rt_a = Global::with_pool(OneTl);
        let rt_b = Global::with_pool(OneTl);
        a = rt_a.handle();
        b = rt_b.handle();
        assert_ne!(rt_a.handle(), rt_b.handle());
    }

    {
        let rt_a = Global::with_pool(OneTl);
        let rt_b = Global::with_pool(OneTl);
        assert_ne!(rt_a.handle(), rt_b.handle());
        assert_eq!(rt_a.handle(), b);
        assert_ne!(rt_b.handle(), a);
        assert_ne!(rt_b.handle(), b);
    }
}
