use parsely_rs::*;

#[derive(ParselyWrite)]
struct Foo {
    one: bool,
    two: u3,
}

fn main() {
    let mut bits_mut = BitsMut::new();
    let foo = Foo {
        one: true,
        two: u3::new(4),
    };

    Foo::write::<NetworkOrder>(&foo, &mut bits_mut, ()).unwrap();
}
