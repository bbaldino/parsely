use parsely::*;

#[derive(ParselyRead)]
struct Foo {
    has_value: bool,
    #[parsely_read(when = "has_value")]
    value: Option<u7>,
}

fn main() {
    let data = vec![0b1_0000001, 2];
    let mut cursor = BitCursor::from_vec(data);

    let foo = Foo::read::<parsely::NetworkOrder, _>(&mut cursor, ()).expect("successful parse");
    assert_eq!(foo.value, Some(u7::new(1)));
}
