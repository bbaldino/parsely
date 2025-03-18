use parsely::*;

#[derive(ParselyRead)]
struct Foo {
    one: bool,
}

fn main() {
    let data = vec![0b10101010];
    let mut cursor = BitCursor::from_vec(data);

    let foo = Foo::read::<parsely::NetworkOrder>(&mut cursor, ()).expect("successful parse");
    assert!(foo.one);
}
