# non-lexical lifetimes (NLL)

> [RFC 2094: NLL](https://rust-lang.github.io/rfcs/2094-nll.html)

non-lexical lifetimes are lifetimes that are based on the control-flow graph, rather than lexical scopes

## 背景

1. https://stackoverflow.com/questions/50251487/what-are-non-lexical-lifetimes
2. https://stackoverflow.com/questions/32300132/why-cant-i-store-a-value-and-a-reference-to-that-value-in-the-same-struct

## 深入

borrow checker is that values may not be mutated or moved while they are borrowed

## 发布

https://blog.rust-lang.org/2018/12/06/Rust-1.31-and-rust-2018.html#non-lexical-lifetimes

