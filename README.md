fluid-let
=========

[![TravisCI: Build Status](https://travis-ci.org/ilammy/fluid-let.svg?branch=master)](https://travis-ci.org/ilammy/fluid-let)
[![AppVeyor: Build status](https://ci.appveyor.com/api/projects/status/77u53qlos7rfjj5p?svg=true)](https://ci.appveyor.com/project/ilammy/fluid-let)

**fluid-let** implements _dynamically-scoped_ variables.

Dynamic or _fluid_ variables are
a handy way to define global configuration values.
They come from the Lisp family of languages
where they are relatively popular for this use case.

## Why may I need this?

Normally the configuration can be kept locally:
as a struct field, or passed via a method argument.
However, sometimes that's not possible (or feasible)
and you need a global configuration.
Dynamic variable binding provides a convenient way
to access and modify global configuration variables.

A classical example would be
configuration of the `Debug` output format.
Suppose you have a `Hash` type for SHA-256 hashes.
Normally you're not really interested in all 32 bytes
of the hash value for debugging purposes,
thus the `Debug` implementation outputs a truncated value:
`Hash(e3b0c442...)`

But what if at _some places_ you need a different precision?
(Or three of them?)
The usual approach would be to introduce a _wrapper type_:

```rust
pub struct DifferentHash(Hash);
```

for which you implement `Debug` differently.
However, that's not very convenient
and sometimes not even possible
as you need to use _the_ `Hash` type
or do not have access to its internals.

Dynamically-scoped variables provide an alternative.
First you define the configuration value:

```rust
fluid_let!(pub static DEBUG_FULL_HASH: bool);
```

Then use it in your `Debug` implementation:

```rust
impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let full = DEBUG_FULL_HASH.get(|current| current.unwrap_or(false));

        write!("Hash(")?;
        if full {
            for byte in self.value[..] {
                write!("{:02X}", byte);
            }
        } else {
            for byte in self.value[..8] {
                write!("{:02X}", byte);
            }
            write!("...")?;
        }
        write!(")")
    }
}
```

Now your users can configure the truncation dynamically:

```rust
DEBUG_FULL_HASH.set(true, || {
    //
    // Code that requires full precision of hashes 
    //
});
```

If they do not configure anything
then default settings will be used. 

## License

The code is licensed under **MIT license** (see [LICENSE](LICENSE)).
