use parsely_rs::*;

#[derive(ParselyRead, ParselyWrite)]
struct Foo {
    // Closures can return a raw value...
    #[parsely_read(map = "|v: u8| { v.to_string() }")]
    // ...or a Result<T, E> as long as E: Into<anyhow::Error>
    #[parsely_write(map = "|v: &str| { v.parse::<u8>() }")]
    value: String,
}

fn main() {
    let mut bits = Bits::from_static_bytes(&[42]);

    let foo = Foo::read::<_, NetworkOrder>(&mut bits, ()).expect("successful parse");
    assert_eq!(foo.value, "42");

    let mut bits_mut = BitsMut::new();

    let foo = Foo {
        value: String::from("42"),
    };

    foo.write::<_, NetworkOrder>(&mut bits_mut, ())
        .expect("successful write");
    let mut bits = bits_mut.freeze();
    assert_eq!(bits.get_u8().unwrap(), 42);
}
