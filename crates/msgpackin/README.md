# msgpackin

Msgpackin pure Rust MessagePack encoding / decoding library.

This crate:
- supports `no_std`, but requires the `alloc` crate. If you're looking
  for no alloc support, see the `msgpackin_core` crate
- supports reading / writing to byte arrays / `Vec<u8>`s in `no_std` mode
- supports reading / writing to `std::io::{Read, Write}` in `std` mode
- supports async reading / writing with the optional `futures-io` feature
- supports async reading / writing with the optional `tokio` feature
- supports `serde` in `no_std` mode
- supports `serde` in `std` mode. (note, for now, you must also enable
  the `serde_std` feature as a workaround until weak dependency features)

License: Apache-2.0
