#[allow(unused_imports)]
use parsely_impl::parsely_read::ParselyRead;
use parsely_macro::ParselyRead;

#[derive(ParselyRead)]
#[parsely(hello = "world")]
struct Foo {
    one: bool,
}

fn main() {}
