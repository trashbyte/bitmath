# bitmath

Tools for arbitrary-width bitwise arithmetic.

Unstable. Use at your own risk.

### `Bits<N>`

The heart of `bitmath`. `Bits` is a generically sized bit vector with support for bitwise operations and arithmetic (WIP).

For example, you can add a pair of 3-bit numbers and find out whether it overflowed:

```rs
let a = Bits::<3>::try_from("101").unwrap();
let b = Bits::<3>::try_from("110").unwrap();
let (result, overflowed) = a.unsigned_add(b);
println!("result: {}, overflowed: {}", result, overflowed);
// result: Bits<3>{ 011 | dec 3/3 | hex 0x3/0x3 }, overflowed: true
```

Or you can take a subset of `Bits` using conventional bitwise syntax using the `bitslice!` macro:
```rs
let word = Bits::<16>::try_from("1011 0001 0110 1011").unwrap();
let high_byte = bitslice!(word[15:8]);
let low_byte = bitslice!(word[7:0]);
println!("{}\n{}\n{}", word, high_byte, low_byte);
// Bits<16>{ 1011 0001 0110 1011 | dec 45419/-20117 | hex 0xb16b/-0x4e95 }
// Bits<8>{ 1011 0001 | dec 177/-79 | hex 0xb1/-0x4f }
// Bits<8>{ 0110 1011 | dec 107/107 | hex 0x6b/0x6b }
```