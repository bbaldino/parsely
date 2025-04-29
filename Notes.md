# Development Notes

Random notes/thoughts while developing.

### byte order modeling

At first I had the ParselyRead trait look like this:

```rust
pub trait ParselyRead<Ctx>: Sized {
    fn read<T: ByteOrder, B: BitRead>(buf: &mut B, ctx: Ctx) -> ParselyResult<Self>;
}
```

but then I realized that when reading a struct, I had to specify the ByteOrder,
even though I planned to annotate that on the struct itself.  I guess it could
be argued that doing it on the struct _technically_ doesn't make sense, since
you could read a given struct from multiple places?  So maybe it _should_ be on
the read call, and then all other values inherit it?

If we go this direction, then I don't think having a byteorder attribute at the
parsely level makes sense?  And the trait definition can stay as it is.

--> Decided to go with this: it makes the most sense for the byteorder to be
determined at call time, not at definition time.  The byteorder will have to be
passed to the initial read call and then will propagate to all nested calls.

### 'assertion' attribute

I originally had a 'fixed' attribute that required a field be set to a specific
value, but I saw that I'd also had an 'assertion' attribute which seems to
cover the 'fixed' use case plus more (supporting not only a specific value, but
other types of assertions).  My original idea for 'fixed' might have been that
it would automatically use that fixed value for writing, but the field would
still need to be set in the struct, so I'm not sure it adds much.  We can still
enforce the assertion on write as well, so will try just 'assertion' for now.

### passing through context values

I usually format packet "layers" like this:

```
struct Header { ... }

struct SomePacket { 
  header: Header,
  ...other fields
}
```

This makes it easy to progressively parse them: parse the header to figure out
how to parse the rest of the payload, etc. But this means that part of a struct
has already been parsed/comes from somewhere else.  So, in addition to using
fields of that previously-parsed piece as context, it also needs to know not to
try and read that field from the buffer.

Should this try and leverage the more generic 'custom_reader' type attribute
(to come later)?  Maybe we can avoid that altogether and just defer to newtypes
and custom impls of ParselyRead for them?  Either way, I'm not sure we'd want
to re-use that for this use case, since that will return a result and this is
intended to be more of a direct "pass-through" assignment.  Could this even
require an Ident, not an Expr?  If it's set around this idea of 'from_context',
then that would be fine.

Note on the current implementation of this: for consistency it wraps the
assignment in an 'Ok' and then the same 'with_context' that would be added to
any field read is added as well, even though it's totally redundant in this
case (because it's just a simple assignment).  They _could_ be given special
treatment and not have that added, but that'd require a unique code path and,
for now at least, I think it'd be better to keep things aligned.

### 'mapping' a field

My initial use case for a way to map a field (i.e. read a different type than
what's defined in the struct and then apply some mapping to it to arrive at the
final type) was for `String` fields in structs.  I took some inspiration from
Deku's implementation but still found it didn't work very well for reading a
`String` as a `Vec<u8>` and then mapping it because reading `Vec<u8>` itself
takes a fair bit of special handling (including additional attributes like
'count').  Deku appears to punt on this and recommends using custom readers for
these cases, which seems like it may make sense.  Will probably play with some
implementations of that and see how they feel.

An interesting scenario came up with regards to the 'write' map path vs the 'read':

Both sides want to take advantage of type inference to avoid having to make the
caller explicitly state types.  Assuming a struct like this:

```rust
#[derive(ParselyRead, ParselyWrite)]
struct Foo {
    #[parsely_read(map = "|v: u8| { v.to_string() }")]
    #[parsely_write(map = "|v: &str| { v.parse::<u8>() }")]
    value: String,
}
```

the generated read code can look like:

```rust
  let original_value = ::parsely_rs::ParselyRead::read::<T>(buf, ()).with_context(...)?;
  (|v: u8| { v.to_string() })(original_value)
      .with_context(...)?;

```

In this case the `read` map function isn't ambiguous about its type, but even
if it was it would be inferred ok because eventually we're going to assign it
to the field which _does_ have an explicit type, so it can be inferred from
there.  The write side is trickier, but mainly because i didn't want to have to
force the caller to explicitly return a result, i wanted to be able to handle a
function that returned either a raw value or a result that could be converted
into a ParselyResult.  The write code looks like this:

```rust
  let mapped_value = (|v: &str| { v.parse() })(&self.value)
      .with_context(...)?;
  ::parsely_rs::ParselyWrite::write::<T>(&mapped_value, buf, ())
      .with_context(...)?;
```

We need to somehow wrap the result of the function in such a way that we end up with a `ParselyResult<T>` regardless of whether it returned some raw type T or some kind of Result<T, E>.

The first solution that comes to mind to try and do that is this:

```rust
pub trait IntoParselyResult<T> {
    fn into_parsely_result(self) -> ParselyResult<T>;
}

impl<T, E> IntoParselyResult<T> for Result<T, E>
where
    E: Into<anyhow::Error>,
{
    fn into_parsely_result(self) -> ParselyResult<T> {
        self.map_err(Into::into)
    }
}

impl<T> IntoParselyResult<T> for T {
    fn into_parsely_result(self) -> ParselyResult<T> {
        Ok(self)
    }
}
```

The problem is that for the case where the expression returns `Result<T, E>`
there's ambiguity: it can't tell if what we want is `ParselyResult<T>` or
`ParselyResult<Result<T, E>>` so the compiler complains about both
implementations being possible.  I tried a bunch of different wrappers and
macros to try and work around this but couldn't come up with anything.  Finally
my only idea was to tweak the implementation for `T` slightly:

```rust
impl<T> IntoParselyResult<T> for T where T: ParselyWrite {
    fn into_parsely_result(self) -> ParselyResult<T> {
        Ok(self)
    }
}
```

Since we don't impl `ParselyWrite` for any `Result` type (and I don't think
we'd have a need to) this disambiguates the two cases enough that there's no
issue trying to infer which one touse.

In order to do this, though, we'll need to change the definition of
`ParselyWrite` to use associated types instead of generics, but I think that
may actually make more sense anyway.

is there an inherent "ordering" to all the attributes that we could always apply in a consistent way?

context reads (assigning the context tuple variables into named variables defined by the 'required_context' attribute): I think these can always be first

determining the type we'll actually read from the buffer: this will be the 'nested' type if its an Option or a Vec, for example.

context values that need to be passed to the read method of the type: this just needs to happen before the read call

generating the actual read call: this needs to know the read type and the context and if it's a collection

fixed values: if the fixed attribute is present, this needs to be appended to the read call in an and_then block

error context: this needs to be added at the very end of the read call

optional field: if the field is optional, then the entire read call above gets wrapped in a conditional block that checks if the when clause passes

Reader {
  plain {
    type
  },
  collection {
    inner_type,
    count
  },
  assign {
    value
  },
  reader_func {
    ident
  },
  map {
    mapper
  }
}

wrapper {
  option {
    when,
    Reader,
  },
  assertion {
    assertion_expr
  }
}

plain read:
  <type>::read(...)

collection read:
  (0..COUNT_EXPR).map(|idx| {
    PLAIN_READ.with_context(|| idx...)
  })

option read:
  if WHEN_EXPR {
    Some(PLAIN_READ.with_context(...))
  } else {
    None
  }

assign from:
  Ok(value)

reader:
  READER::read(...)

// mapping is particularly special because we never explicitly identify the
// original type or the mapped type: we let the mapper function work to infer
// those for us so we can avoid that.  (technically knowing the mapped type
// isn't hard, as it's the type of the field...)
map:
  let original = ParselyRead::read(...).with_context(...)
  let mapped = (MAPPER)(original).with_context(...)
  Ok(mapped)

assertion:
  currently the assertion is done as an and_then and appended onto whatever the output is (but _before_ the 'option' wrapper is added)

### 'Linked' fields

Often times two fields are related, e.g. a length field and its corresponding
vector.  Sometimes these are two "peer" fields in the same type, and sometimes
they're spread across types (for example: the length field in an rtcp header is
a function of the size of the rtcp payload).  I believe this link should impact
both reading and writing:

When reading, it might influence/impact what should be considered the "end" of
a buffer when trying to read the rest of a payload (even though the actual
buffer may go beyond it, i.e. in the case of compound rtcp or a different
field).

When writing, it needs to go the other direction: the size of some payload
needs to be reflected "upward" into a length field.

For example, an RTCP BYE packet:

When reading:

First we read the header.  The length value from the header should be used to
create a bounded subcursor to be used when reading the payload, this way the
payload reading doesn't risk reading beyond where it should.  In this case,
this would take place "outside" of Parsely, since those two operations are
independent.  For 'peer' fields this is also already covered by something like
the 'count' attribute.  Will see if other cases emerge that require different
handling.

When writing:
Before writing the header, the length field should be sync'd to reflect the
proper length of the packet.  This makes me think we should prevent/discourage
this field from being written to manually.  Should parsely define method to
access fields as well?

We _could_ do this using context for the write path:

```rust
#[parsely_write(required_context("payload_length_bytes: u32"))]
struct RtcpHeader { 
  ...
  // Ignore the current value of length_field (_v) completely and don't even
  // touch the field: just // use the map function as a hook to calculate the
  // value that should be written
  #[parsely_write(map("|_v: &u16 | payload_length_bytes / 4 + 1))]
  length_field: u16
}

struct RtcpByePacket {
  #[parsely_write(context("self.length_bytes()"))]
  header: RtcpHeader,
  ...
}
```

This feels a little weird, since there's already an existing field that models
this length, though.  And I'm not sure how we'd take that context into account
when writing `length_bytes`, using `map` maybe?

```rust
#[parsely_write(sync_args("payload_length_bytes: u32"))]
struct RtcpHeader { ... }

impl RtcpHeader {
  fn sync<(u32,)>(&mut self, (payload_length_bytes,): (u32,)) {
    self.length_bytes = payload_length_bytes / 4 + 1;
  } 
}

struct RtcpByePacket {
  #[parsely_write(sync("self.length_bytes()"))]
  header: RtcpHeader,
  ...
}
```

This is another option, where we denote that a field needs updating before
writing, so the `ParselyWrite` impl would first call `sync` on that field with
the given arguments before calling write on it.

This feels better, I think.  It does mean manual implementations have to
remember to call sync, whereas doing it via the write context guarantees it
would need to be passed, though.

--> My initial thought is that, although a separate method call means there's
something to possibly miss, the 'sync' style feels better, so going to give
that a shot.

So I took a look at this and it ends up still being pretty sticky.  One thing I
ran into is that `ParselyWrite::write` currently takes `&self`, but if `write` is
going to automatically call `sync` then it'd need `&mut self` and I'm not sure
I want to do that.  So maybe the call to `sync` will have to be manual and it
will just be generated to do the right things?  Going with that for now.

TODO:
  can we get rid of the ambiguity of a plain "Ok" in sync_func? Could we make
it such that plain (no Ok needed) would also work?

Follow-up on this: I think that, when writing, the generated code should call
'sync' on every field, that way if the user forgets to set the 'sync_with'
attribute it'll cause a compile error (since the correct types won't be passed)
which will help eliminate some bugs.  For this to work, though, _every_ type
needs a sync method (including the built-ins) so we need a trait for this that
can be implemented for them.  Should this function be part of the ParselyWrite
trait? Or something separate? It'd be required for every ParselyWrite tpe, so
maybe just doing it in ParselyWrite is better?  But then the sync_with args
would need to be part of the ParselyWrite generics, which isn't great...

...Do we ever use Ctx when writing?  Could the sync type be the equivalent of
the context for writing?  I don't think context is needed for writing (I've
seen no use cases of it so far, and it seems like any write context), but
logically it seems like a use-case could exist for it...

Follow-up:
I've separated out the sync code into a trait (`StateSync`).  I've also moved
the arguments to an associated type, which means we can make `StateSync` a
supertrait of `ParselyWrite`.  The idea is that, when generating the write,
`sync` will be called on every field so there's no chance of it being missed.

One issue with this is built-in types: they need a default impl so that the
compiler doesn't complain about calling `sync` on them.  But sometimes we need
to sync fields that are built-in types.  For example, the length field of the
RtcpHeader needs to be sync'd using the sync args.

There are 3 aspect to synchronizing fields:

* `sync_args`: This is attached to a type definition and it describes the names
and types of the arguments that will be expected in that type's `StateSync`
impl.
* `sync_with`: This is attached to a field, and describes the values that will be
used when calling that field's `sync` method.
* `sync_func`: This is attached to a field and describes how _that field_
should be updated in the `sync` method for its surrounding type using the args
passed to it via `sync_args`

The last one is confusing.  Maybe it's better named 'sync_expr'?

### The buffer type

Currently, ParselyRead takes any buffer that is BitRead and ParselyWrite takes
any buffer that is BitWrite.  The issue here is that BitRead and BitWrite are
very limited: they can't seek, they can't create sub-buffers, they can't tell
how many bytes are remaining.  It would be nice to allow crates ways to add
additional trait bounds on the buffer type so that they can use their own types
that implement BitRead/BitRead but might also provide other functionality.

This might look like:

```rust
#[derive(ParselyRead)]
#[parsely(buffer_type = "BitRead + Seek + Sliceable + Sized")]
struct MyStruct { ... }
```

where those values would be added as trait bounds in the ParselyRead impl, e.g.:

```rust
imp ParselyRead<()> for MyStruct {
  fn read<T: ByteOrder, B: BitRead + Seek + Sliceable + Sized>(buf: &mut B) -> ParselyResult<Self> { ... }
}
or, obviously, you could create a trait to encompass those:

```rust
trait PacketBuffer: BitRead + Seek + Sliceable + Sized { ... }
...

#[derive(ParselyRead)]
#[parsely(buffer_type = "PacketBuffer")]
struct MyStruct { ... }
```

Some thoughts:

* I think a crate would have to be very consistent with their use of this: mixing and matching could lead to problems
* Is it possible to write an 'alias' for a macro attribute?

### Post hooks

Now that custom buffers can be set, we're in better shape to handle things like
consuming/adding padding.  I was trying to think about how we might do that
from a parsely perspective:

At first I was thinking of a `padded` attribute which would denote that padding
should be consumed after reading a field and added after writing a field, but
the core parsely-code only knows about BitRead/BitWrite, so it can't do padding
on its own.

So that means we need something more generic.  My next thought was a "post
hook" that would allow the caller to specify some operation that should happen
after the read/write.  For example:

```rust
#[derive(ParselyRead, ParselyWrite)]
#[parsely_read(buffer_type = "PacketBuffer")]
#[parsely_write(buffer_type = "PacketBufferMut")]
struct MyStruct {
  length: u8,
  #[parsely_read(count = "length", post = "buf.consume_padding()")]
  #[parsely_write(post = "buf.add_padding()")]
  data: Vec<u8>,
}
```

Where "consume_padding" and "add_padding" would be features of some custom
buffer type (PacketBuffer/PacketBufferMut here).

Maybe it should be "after"?

--> After the transition to Buf/BufMut, I think we can pull off handling
padding at the parsely/attribute level.  When reading, we can take the number
of remaining bytes in the buffer at the start of the read and then after it's
done and then compute how much padding we need to get aligned.  It assumes the
buffer _starts_ aligned, but I think that's reasonable.

It's similar on the write side: take the remaining amount at the start and at
the end and add any padding necessary.

### Custom reader/writer functions

I had these originally, but don't think they feel necessary as opposed to just
doing a custom ParselyRead/ParselyWrite impl.  This approach will require a
newtype in some cases where maybe you wouldn't otherwise have one, but at this
point I don't think it feels worth it.

### Separate 'read/write' and 'read/write_with_context'

I notice I'm getting pretty tired of having to pass an empty tuple in places
where no context is needed.  Could we do 2 methods, where if no context is
needed it could be called without a context arg?  I wouldn't want a type that
_does_ need context to be able to call the one without it, though.  Maybe a
special extension trait that's implemented for only ParselyRead<()>?  But I
have the buffer type in the trait now too, and specifying that might not be
doable?  The extension trait impl would have to be dynamically generated per
type, maybe?

### 'while' attribute for collections

I needed this in rtp to be able to keep reading until a buffer is empty, but
after doing the code I realized I had no good way to test it because the
built-in buf doesn't provide anything that could be used with it, which makes
me think putting it in parsely might not make sense.  This use case _could_
have used the custom reader attribute, maybe?

### Using ParselyRead/ParselyWrite with types other than BitBuf/BitBufMut

For things like RTP parsing, we don't want to use the BitBuf/BitBufMut
abstractions because they limit the efficiency we can get when we use
Bits/BitsMut directly.  It would be nice to be able to leverage
ParselyRead/ParselyWrite for other types as well: i.e. manually implementing it
for the RTP packet types.  The problem is that the trait bound
(BitBuf/BitBufMut) is built into the trait, so we can't really do that.  I
looked at changing the "buffer" type to be an associated type, but that has a
couple problems:

* We can't do `type Buf = impl BitBuf` or `type Buf = dyn BitBUf` so we lose
the ability to say "some BitBuf type".  We _could_ add another layer of
indirection here, but that's a bit of a bummer.
* Supporting multiple buffer types also makes me think that it'd be nice to be
able to provide different/optimized read/write impls for different types (e.g.
one for BitBuf and another when we know it's a Bits specifically or something)

--> look into a refactoring of the read/write traits to accomplish this
