use parsely_rs::*;

#[derive(ParselyRead, ParselyWrite)]
struct Foo(u8, u8);

fn main() {
    let mut bits = Bits::from_static_bytes(&[42, 43]);

    let foo = Foo::read::<NetworkOrder>(&mut bits, ()).expect("successful parse");
    assert_eq!(foo.0, 42);
    assert_eq!(foo.1, 43);

    let mut bits_mut = BitsMut::new();
    foo.write::<NetworkOrder>(&mut bits_mut, ())
        .expect("successful write");
    let bytes = bits_mut.chunk_bytes();
    assert_eq!(bytes[0], 42);
    assert_eq!(bytes[1], 43);
}
