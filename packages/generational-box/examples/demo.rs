use generational_box::GenerationalBox;

fn main() {
    let item = GenerationalBox::<i32>::new(10);

    let r = item.read();

    drop(r);

    let o = item.write();

    dbg!(&o);

    // dbg!(&r);

    drop(o);

    item.dispose();

    item.set(20);

    let r = item.read();

    dbg!(&r);
}
