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
