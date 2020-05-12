use pui::{make_global_pool, runtime::Global};

make_global_pool! {
    stack _Stack(Global);
}

make_global_pool! {
    thread_local stack _StackTl(Global);
}

make_global_pool! {
    queue _Queue(Global);
}

make_global_pool! {
    thread_local queue _QueueTl(Global);
}

make_global_pool! {
    one _One(Global);
}

make_global_pool! {
    thread_local one _OneTl(Global);
}
