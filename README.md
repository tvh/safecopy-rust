# safecopy-rust

This is a port of SafeCopy (https://github.com/acid-state/safecopy) to rust.

# Questions

### Why?

I started this project ages ago as a way to learn Rust.
I knew SafeCopy and it sounded like a good idea at the time.
More recently, I added automatic derivation of the SafeCopy trait via a proc macro.

### Should I use this?

Probably not. The proc macro is not stable or finished yet. Also, more generally, SafeCopy doesn't make for a good serialization format.
The versioning tags end up taking up _a lot_ of space.
Also, you will have to maintain all prior versions of your data structures without a clear path to ever remove them.

### What to use instead?

If you want a good and migratable binary serialization format, I would use protocol-buffers via the [prost](https://github.com/tokio-rs/prost) crate.