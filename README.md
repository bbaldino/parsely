# Parsely

Convenient type serialization and deserialization in Rust for binary formats.

Parsely uses derive macros to automatically implement serialization and
deserialization methods for your types.

This crate is heavily inspired by the [Deku](https://docs.rs/deku/latest/deku/)
crate (and is nowhere near as complete).  See [Differences from
Deku](#differences-from-deku) below.

## Example

Say you want to parse an [RTCP header](https://datatracker.ietf.org/doc/html/rfc3550#section-6.1
) formatted like so:

```ignore
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|V=2|P|    SC   |      PT       |             length            |
+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+
```

Where the version (V) field should always contain the value `2`. The code to
serialize and deserialize it can be written with Parsely like this:

```rust
# use parsely_rs::*;

#[derive(Debug, PartialEq, Eq, ParselyRead, ParselyWrite)]
pub struct RtcpHeader {
    #[parsely(assertion = "|v: &u2| *v == 2")]
    pub version: u2,
    pub has_padding: bool,
    pub report_count: u5,
    pub packet_type: u8,
    pub length_field: u16,
}

// Reading it from a buffer
fn do_read(data: Vec<u8>) {
  let mut bits = Bits::from_owner_bytes(data);

  let rtcp_header = RtcpHeader::read::<NetworkOrder>(&mut bits, ())
    .context("Reading RtcpHeader")
    .unwrap();
}

// Writing it out to a buffer
fn do_write(rtcp_header: RtcpHeader) {
  let mut bits_mut = BitsMut::new();
  
  let result = rtcp_header.write::<NetworkOrder>(&mut bits_mut, ());
}
```

## Traits

The `ParselyRead` trait is used for reading data from a buffer.  `ParselyRead`
can be derived and its logic customized via the attributes described below, but
can also be manually implemented.

```rust
# use parsely_rs::*;

pub trait ParselyRead<B>: Sized {
    type Ctx;
    fn read<T: ByteOrder>(buf: &mut B, ctx: Self::Ctx) -> ParselyResult<Self>;
}
```

The `ParselyWrite` trait is used for writing data to a buffer.  Like
`ParselyRead`, `ParselyWrite` can be derived and customized or manually
implemented.

```rust
# use parsely_rs::*;

pub trait ParselyWrite<B>: StateSync + Sized {
    type Ctx;
    fn write<T: ByteOrder>(&self, buf: &mut B, ctx: Self::Ctx) -> ParselyResult<()>;
}
```

The `StateSync` trait is a required supertrait of `ParselyWrite` and enforces
synchronization of fields before writing.

```rust
use parsely_rs::*;

pub trait StateSync: Sized {
    type SyncCtx;

    fn sync(&mut self, sync_ctx: Self::SyncCtx) -> ParselyResult<()> {
        Ok(())
    }
}
```

When deriving `ParselyWrite`, a `StateSync` implementation will be generated as
well.  See the [dependent fields section](#dependent-fields) for more
information on how attributes can be used to customize the behavior.  If you
manually implement `ParselyWrite` yourself, you'll need to implement
`StateSync` as well.  If the field requires no synchronization, you can use the
`impl_stateless_sync` macro to generate a default impl for your type.

Sometimes serializing or deserializing a type requires additional data that may
come from somewhere else.  The `Ctx` generic can be defined as a tuple and the
`ctx` argument can be used to pass additional values.

See the [Context and required context](#context-and-required-context) section
below for more information.

The `ByteOrder` generic is used to describe how the data is laid out in the
buffer (e.g. LittleEndian or BigEndian).  The `B` generic is the buffer type.
Typically this is an instance of `BitBuf` for reading and `BitBufMut` for
writing.  Both types come from the
[bit-cursor](http://github.com/bbaldino/bitcursor) crate.

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
use parsely_rs::*;

#[derive(Debug, ParselyRead, ParselyWrite)]
pub struct MyStruct {
  #[parsely(assertion = "|v: &u8| *v == 42")]
  pub value: u8,
}
```

```rust
use parsely_rs::*;

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
or a closure which takes a type `T` by value where `T: ParselyRead` and can
return either a type `U` or a `Result<U, E>` where `U` is the type of
the field and `E: Into<anyhow::Error>`.

When passed via `#[parsely_write]`, the argument must evaluate to a function
or closure which takes a reference to a type `T`, where `T` is the type of
the field and returns either a type `U` or a `Result<U, E>` where
`U: ParselyWrite` and `E: Into<anyhow::Error>`.

| Mode | Available |
| --------- | -------- |
| `#[parsely]` | :x: |
| `#[parsely_read]` | :white_check_mark: |
| `#[parsely_write]` | :white_check_mark: |

#### Examples

<details>
  <summary>Click to expand</summary>

This example has a `String` field but reads a `u8` from the
buffer and converts it.  On write it does the opposite.  

```rust
use parsely_rs::*;

#[derive(ParselyRead, ParselyWrite)]
struct Foo {
    // Closures can return a raw value...
    #[parsely_read(map = "|v: u8| { v.to_string() }")]
    // ...or a Result<T, E> as long as E: Into<anyhow::Error>
    #[parsely_write(map = "|v: &str| { v.parse::<u8>() }")]
    value: String,
}

let mut bits = Bits::from_static_bytes(&[42]);

let foo = Foo::read::<NetworkOrder>(&mut bits, ()).expect("successful read");
assert_eq!(foo.value, "42");

let mut bits_mut = BitsMut::new();
foo.write::<NetworkOrder>(&mut bits_mut, ()).expect("successful write");
assert_eq!(bits_mut.freeze(), Bits::from_static_bytes(&[42]));
```

</details>

### Count

When reading a `Vec<T>`, we need to know how many elements to read.  The `count`
attribute is used to describe how many elements should be read from the buffer.

Any expression that evaluates to a number that can be used in a range
expression can be used.

| Mode | Available |
| --------- | -------- |
| `#[parsely]` | :x: |
| `#[parsely_read]` | :white_check_mark: |
| `#[parsely_write]` | :x: |

#### Examples

<details>
  <summary>Click to expand</summary>

Here a `u8` is read into the `data_size` field and the value of that field is
used to denote the number of elements.

```rust
use parsely_rs::*;

#[derive(ParselyRead, ParselyWrite)]
struct Foo {
    data_size: u8,
    // Here we refer to the previously-read 'data_size' field to describe the length
    #[parsely_read(count = "data_size")]
    data: Vec<u8>,
}

```

</details>

### When

Optional fields need to be given a predicate that describe when they should be
attempted to be read. The `when` attribute takes an expression that evaluates
to a boolean.  A result of true means the field will be read from the buffer,
false means it will be skipped and set to `None`.

| Mode | Available |
| --------- | -------- |
| `#[parsely]` | :x: |
| `#[parsely_read]` | :white_check_mark: |
| `#[parsely_write]` | :x: |

#### Examples

<details>
  <summary>Click to expand</summary>

Here, a boolean value is read into the `has_value` field and whether a `u32` is
read for `value` field is based on if `has_value` is true or false.

```rust
use parsely_rs::*;

#[derive(ParselyRead, ParselyWrite)]
struct Foo {
    has_value: bool,
    // Here we refer to the previously-read 'has_value' field to 
    // describe whether or not this field is present
    #[parsely_read(when = "has_value")]
    value: Option<u32>,
}

```

</details>

### Assign from

Sometimes a field should be assigned to a value rather than read from the
buffer.  Any expression evaluating to the type of the field can be passed.

| Mode | Available |
| --------- | -------- |
| `#[parsely]` | :x: |
| `#[parsely_read]` | :white_check_mark: |
| `#[parsely_write]` | :x: |

#### Examples

<details>
  <summary>Click to expand</summary>

Here the `header` value has already been read and is passed in via context.  It
is then assigned directly to the `header` field.

```rust
use parsely_rs::*;

#[derive(ParselyRead, ParselyWrite)]
struct Header {
  payload_type: u8,
}

#[derive(ParselyRead, ParselyWrite)]
#[parsely_read(required_context("header: Header"))]
struct Packet {
  #[parsely_read(assign_from = "header")]
  header: Header,
  other_field: u8,
}
```

</details>

### Dependent fields

Often times packets will have fields whose values depend on other fields.  A
header might have a length field that should reflect the size of a payload.
`Parsely` defines multiple attributes to define these relationships:

The `sync_args` attribute is used on a struct to define what external
information is needed in order to sync its fields correctly.

The `sync_expr` attribute is used on a specific field to define how it should
use the values from `sync_args` (or elsewhere) in order to sync.

The `sync_with` attribute is used to pass information to a field to synchronize
it.

All types that implement `ParselyWrite` must also implement the `StateSync` trait.

The `sync` function from the `StateSync` trait should be called explicitly
before writing the type to a buffer to make sure all fields are consistent.
| Mode | Available |
| --------- | -------- |
| `#[parsely]` | :x: |
| `#[parsely_read]` | :x: |
| `#[parsely_write]` | :white_check_mark: |

#### Examples

<details>
  <summary>Click to expand</summary>

Here, a header contains a length field that should describe the length of the
entire packet.  The payload contains a variable-length array, so its length
needs to be taken into account rest of the payload.  A field from the header is
passed as context to the payload parsing.

```rust
use parsely_rs::*;

#[derive(Debug, ParselyWrite)]
// sync_args denotes that this type's sync method takes additional 
// arguments.  By default a type's sync field takes no arguments
#[parsely_write(sync_args("payload_length_bytes: u16"))]
struct Header {
    version: u8,
    packet_type: u8,
    // sync_func can refer to an expression or a function and will be used to
    // update the annotated // field, it should evaluate to ParselyResult<T> 
    // where T is the type of the field.  You can // refer to variables defined in
    // sync_args.
    #[parsely_write(sync_expr = "ParselyResult::Ok(payload_length_bytes + 4)")]
    length_bytes: u16,
}

#[derive(Debug, ParselyWrite)]
struct Packet {
    // sync_with attributes add lines to this type's sync method to call 
    // sync on its fields (and what arguments to pass)
    #[parsely_write(sync_with("self.data.len() as u16"))]
    header: Header,
    data: Vec<u8>,
}

let mut packet = Packet {
    header: Header {
        version: 1,
        packet_type: 2,
        length_bytes: 0,
    },
    data: vec![1, 2, 3, 4],
};

packet.sync(()).unwrap();

assert_eq!(packet.header.length_bytes, 8);
```

</details>

### Context and required context

Sometimes in order to read or write a struct or field, additional data is
needed.  Structs can declare what additional data is needed via the
`required_context` attribute.  Additional data can be also be passed down to
fields via the `context` attribute.  Any required_context or previously-parsed
field name can be used.

The argument passed to `required_context` is a comma-separated list of typed
function arguments (e.g. `size: u8, name: String`).  The variable names there
can be used in other attributes.

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

Here, a header is parsed first which contains information needed to parse the
rest of the payload.  A field from the header is passed as context to the
payload parsing.

```rust
use parsely_rs::*;

#[derive(ParselyRead, ParselyWrite)]
struct FooHeader {
  packet_type: u8,
  payload_len: u32,
}

#[derive(ParselyRead, ParselyWrite)]
// Foo needs additional context in order to be parsed from a buffer
#[parsely_read(required_context("len: u32"))]
struct Foo {
    // The required_context variable is accessible and can be referred to when
    // describing the length of the Vec that should be read
    #[parsely_read(count = "len")]
    data: Vec<u8>,
}

fn run(buf: &mut Bits) {
  let foo_header = FooHeader::read::<NetworkOrder>(buf, ()).unwrap();
  // Pass the relevant field from header to the payload's read method
  let foo_payload = Foo::read::<NetworkOrder>(buf, (foo_header.payload_len,)).unwrap();
}


```

</details>

## TODO/Roadmap

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
