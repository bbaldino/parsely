use parsely_rs::*;

#[derive(Debug, ParselyRead, ParselyWrite)]
#[parsely(key_type = "u8")]
enum Foo {
    #[parsely(id = 1)]
    One,
    #[parsely(id = 2)]
    Two(u8),
    #[parsely(id = 3)]
    Three { bar: u8, baz: u16 },
}

fn main() {
    #[rustfmt::skip]
    let mut bits = Bits::from_static_bytes(
        &[
            // First instance: Foo::One, no data
            1,
            // Second instance: Foo::Two, value of 1
            2, 1,
            // Third instance: Foo::Three, { bar: 1, baz: 42 }
            3, 1, 0, 42,
        ]
    );
    let bits_clone = bits.clone();

    let one = Foo::read::<NetworkOrder>(&mut bits, ()).expect("one");
    assert!(matches!(one, Foo::One));
    let two = Foo::read::<NetworkOrder>(&mut bits, ()).expect("two");
    assert!(matches!(two, Foo::Two(1)));
    let three = Foo::read::<NetworkOrder>(&mut bits, ()).expect("three");
    assert!(matches!(three, Foo::Three { bar: 1, baz: 42 }));

    let mut bits_mut = BitsMut::new();
    one.write::<NetworkOrder>(&mut bits_mut, ())
        .expect("successful write one");
    two.write::<NetworkOrder>(&mut bits_mut, ())
        .expect("successful write two");
    three
        .write::<NetworkOrder>(&mut bits_mut, ())
        .expect("successful write three");

    assert_eq!(bits_clone, bits_mut.freeze());
}
