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

    let one = Foo::read::<NetworkOrder>(&mut bits, ()).expect("one");
    assert!(matches!(one, Foo::One));
    let two = Foo::read::<NetworkOrder>(&mut bits, ()).expect("two");
    assert!(matches!(two, Foo::Two(1)));
    let three = Foo::read::<NetworkOrder>(&mut bits, ()).expect("three");
    assert!(matches!(three, Foo::Three { bar: 1, baz: 42 }));

    // TODO: write test: need to fix enum writer to always write tag
    let mut bits_mut = BitsMut::new();
    one.write::<NetworkOrder>(&mut bits_mut, ())
        .expect("successful write one");
}
