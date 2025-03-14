# Parsely

Convenient type serialization and deserialization in Rust.

Parsely uses derive macros to automatically implement serialization and
deserialization methods for your types.

This crate is heavily inspired by the [Deku](https://docs.rs/deku/latest/deku/)
crate (and is nowhere near as complete).  See [Differences from
Deku](#differences-from-deku) below.

## Example

Say you want to parse an [RTCP header](https://datatracker.ietf.org/doc/html/rfc3550#section-6.1
) formatted like so:

```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|V=2|P|    SC   |      PT       |             length            |
+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+
```

Where the version (V) field should always contain the value `2`. The code to
serialize and deserialize it can be written with Parsely like this:

```rust
#[derive(Debug, PartialEq, Eq, ParselyRead, ParselyWrite)]
pub struct RtcpHeader {
    #[parsely(assertion = "|v: &u2| *v == 2")]
    pub version: u2,
    pub has_padding: bool,
    pub report_count: u5,
    pub packet_type: u8,
    pub length_field: u16,
}
```

## Attributes

Parsely defines various attributes to make parsing different structures
possible.  There are 3 modes of applying attributes:

* read + write via `#[parsely]`
* read-only via `#[parsely_read]`
* write-only via `#[parsely_write]`

Some attributes are _only_ available for reading or writing

### Assertion

An assertion is applied to the value pulled from the buffer after reading or to
the field before writing.  They allow reading and/or writing to fail when the
assertion fails.  An assertion can either be a closure or the path to a
function.  Both styles must be functions which take a reference to the value's
type and return a boolean.

| Mode | Available |
| --------- | -------- |
| `#[parsely]` | :white_check_mark: |
| `#[parsely_read]` | :white_check_mark: |
| `#[parsely_write]` | :white_check_mark: |

#### Examples

<details>
  <summary>Click to expand</summary>

```rust
#[derive(Debug, ParselyRead, ParselyWrite)]
pub struct MyStruct {
  #[parsely(assertion = "|v: &u8| *v == 42")]
  pub value: u8,
}
```

```rust
fn my_assertion(value: &u8) -> bool {
  *value == 42
}

#[derive(Debug, ParselyRead, ParselyWrite)]
pub struct MyStruct {
  #[parsely(assertion = "my_assertion")]
  pub value: u8,
}
```

</details>

### Map

A transformation may be applied to a value read from a buffer before assigning
it to the field, or to a field's value before writing it to the buffer.

Because the signatures for read and write map functions are slightly different,
the map attribute must be applied independently for reading and writing via
`#[parsely_read]` and `#[parsely_write]`

When passed via `#[parsely_read]`, the argument must evaluate to a function
which takes a type T by value, where T is `ParselyRead` and should return a
`ParselyResult<U>`, where U matches the type of the field.

When passed via `#[parsely_write]`, the argument must evaluate to a function
which takes a reference to a type T, where T is the type of the field and
returns a `ParselyResult<U>` where U is `ParselyWrite`.

| Mode | Available |
| --------- | -------- |
| `#[parsely]` | :x: |
| `#[parsely_read]` | :white_check_mark: |
| `#[parsely_write]` | :white_check_mark: |

#### Examples

<details>
  <summary>Click to expand</summary>

This (quite contrived) example has a boolean field but reads a u1 from the
buffer and converts it.  On write it does the opposite.  

```rust
#[derive(ParselyRead, ParselyWrite)]
struct Foo {
    #[parsely_read(map = "|v: u1| -> ParselyResult<bool> { Ok(v > 0) }")]
    #[parsely_write(map = "|v: &bool| -> ParselyResult<u1> { Ok(u1::from(*v)) }")]
    one: bool,
}

```

</details>

### Count

### When

### Assign from

### Reader

### Writer

### Context and required context

Sometimes in order to read or write a struct or field, additional data is needed.  Structs can declare what additional data is needed via the `required_context` attribute.  Additional data can be passed to fields via the `context` attribute.

The argument passed to `required_context` is a comma-separated list of typed
function arguments (e.g. `size: u8, name: String`).  

The argument passed to `context` is a comma-separated list of expressions that
evaluate to values that should be passed to that field's read and/or write
method.

| Mode | Available |
| --------- | -------- |
| `#[parsely]` | :white_check_mark: |
| `#[parsely_read]` | :white_check_mark: |
| `#[parsely_write]` | :white_check_mark: |

#### Examples

<details>
  <summary>Click to expand</summary>

Here, additional data is required in order to read `Foo`, and additional data
is passed on to one of its fields:

```rust
#[derive(ParselyRead, ParselyWrite)]
#[parsely_read(required_context("size_one: u32", "size_two: u32"))]
struct Inner {
    #[parsely_read(count = "size_one")]
    data: Vec<u8>,
    #[parsely_read(count = "size_two")]
    data2: Vec<u8>,
}

#[derive(ParselyRead, ParselyWrite)]
#[parsely_read(required_context("size: u32"))]
struct Foo {
    // Use 'size' to determine the number of elements to read into 'data'
    #[parsely_read(count = "size")]
    data: Vec<u8>,
    // Pass additional values to Inner's read method.  Any expression is supported.
    #[parsely_read(context("size / 2", "size / 2"))]
    inner: Inner,
}
```

</details>

## TODO/Roadmap

* Unit/Newtype/Tuple struct and enum support
* Probably need some more options around collections (e.g. `while`)

## Differences from Deku

The original intent for writing this crate was to come up with a
straightforward, generic way to quickly write serialization and deserialization
code for packets.  It does not strive to be a "better Deku": if you're
writing any sort of production code, Deku is what you want.  The goal here
was to have an excuse to play around with derive macros and have a library that
I could leverage for other personal projects.  That being said, here are a
couple decisions I made that, from what I can tell, are different from Deku:

1. The [nsw-types](https://github.com/bbaldino/nsw-types) crate is used to
   describe fields of non-standard widths (u3, u18, u33, etc. as opposed to
using u8, u16, etc. and specifying the number of bits via an attribute), which
makes message definitions more explicitly-typed and eliminates the needs for
extra attributes.  The tradeoff here is that a special cursor type
([BitCursor](https://github.com/bbaldino/bitcursor)) is required to process the
buffer.

1. Byte order is specified as part of the read and write calls as opposed to
   the struct definition.  Deku may support this as well, but I didn't even add
attributes to denote a type's byte order because it felt like that should exist
outside the type's definition.

1. More...
