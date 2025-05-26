use parsely_rs::*;

#[derive(ParselyRead)]
struct Foo(u8, u8);

fn main() {
    let mut bits = Bits::from_static_bytes(&[42, 43]);

    let foo = Foo::read::<NetworkOrder>(&mut bits, ()).expect("successful parse");
    assert_eq!(foo.0, 42);
    assert_eq!(foo.1, 43);
}
