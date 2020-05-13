# finderinfo

A library to parse Apple HFS/HFS+/APFS FinderInfo attribute.

On modern MacOS systems, objects in the filesystem can have an extended attribute called `com.apple.FinderInfo`. This
attribute is 32 bytes long and largely undocumented. It turns out that this attribute is actually the old HFS Finder
Info struct in the first 16 bytes, and the Extended Finder Info struct in the second 16 bytes. This library provides a
mechanism by which a Rust program can programmatically interact with these structures.

This crate also provides an executable `finderinfo`, which is a small utility that can parse and display the contents of
the Finder Info blob. If built with the `xattr` feature, the library is able to read and write the
`com.apple.FinderInfo` extended attribute on MacOS systems.

## Example

```rust
let buf = vec![
    0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8,
    0x40u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8,
    0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8,
    0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8,
];
let finder_info = FinderInfoFolder::read(&mut io::Cursor::new(buf));
println!("{:?}", finder_info);
```
