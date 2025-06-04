use parsely_rs::*;

#[derive(ParselyRead, ParselyWrite)]
struct Foo(u8);

fn main() {
    let mut bits = Bits::from_static_bytes(&[42]);

    let foo = Foo::read::<NetworkOrder>(&mut bits, ()).expect("successful parse");
    assert_eq!(foo.0, 42);

    let mut bits_mut = BitsMut::new();

    foo.write::<NetworkOrder>(&mut bits_mut, ())
        .expect("successful write");
    assert_eq!(bits_mut.chunk_bytes()[0], 42);
}
