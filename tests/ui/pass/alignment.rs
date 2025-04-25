use parsely_rs::*;

#[derive(ParselyRead, ParselyWrite)]
#[parsely(alignment = 4)]
struct Foo {
    one: u8,
}

fn main() {
    let mut bits = Bits::from_static_bytes(&[42, 0, 0, 0]);

    let foo = Foo::read::<_, NetworkOrder>(&mut bits, ()).unwrap();
    assert_eq!(foo.one, 42);
    assert_eq!(bits.remaining_bytes(), 0);

    let mut bits_mut = BitsMut::new();

    Foo::write::<_, NetworkOrder>(&foo, &mut bits_mut, ()).unwrap();
    assert_eq!(bits_mut.len_bytes(), 4);
}
