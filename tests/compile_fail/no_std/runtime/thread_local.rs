use pui::make_counter;

make_counter! {
    threadlocal type _Runtime = u8;
}

fn main() {}
