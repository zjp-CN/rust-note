# FAQ

## 什么是 proc-macro crate

proc-macro crate 指 `Cargo.toml` 中，其 crate type 被定义成如下的 crate：

```toml
[lib]
proc-macro = true
```

它**只能导出过程宏**。

正常的函数、类型、模块、`macro_rules!` 等内容都不能导出，但可以仅在其内部定义和使用。

## 什么是过程宏

- 从形式上看：是带着特定属性的公有函数，其输入为一个或两个 TokenStream，输出是一个 TokenStream
- 从功能上看：是 AST 到 AST 的函数，即从编译器获取和返还 AST。
- 与声明宏的关系：是声明宏的拓展，而非声明宏的替代品

| 类别      | 函数属性                                                                             | 公有函数名              | 函数签名                                    |
|-----------|--------------------------------------------------------------------------------------|-------------------------|---------------------------------------------|
| 函数式    | `#[proc_macro]`                                                                      | 函数名即宏名            | `(TokenStream) -> TokenStream`              |
| derive 式 | `#[proc_macro_derive(Name)]` 或者<br> `#[proc_macro_derive(Name, attributes(attr))]` | 任意，因为宏名是 `Name` | `(TokenStream) -> TokenStream`              |
| 属性式    | `#[proc_macro_attribute]`                                                            | 函数名即宏名            | `(TokenStream, TokenStream) -> TokenStream` |

## 过程宏的优缺点

优点：
1. 过程宏利用了 Rust 强大的静态类型系统的优势，从而以结构化的方式操作 AST
2. 函数式过程宏可以实现与声明宏相同的功能
3. derive 式过程宏和属性式过程宏对使用者更加方便

缺点：
1. 学习成本高：作为过程宏的编写者，你需要对 Rust 非常熟悉（对于过程宏的使用者，学习如何使用过程宏并不难）
2. 增加了编译成本
3. 相比编写声明宏，过程宏的代码量更多（当然，意味着过程宏的功能更丰富）

## derive 宏与属性宏的区别

derive 宏所生成代码的代码是附加性质的，通常生成某类型的 `impl` 代码，尤其是生成某 trait impl 的代码。

而属性宏更加通用和自由，生成的代码可以是附加或者替换性质的。

## 编写过程宏所涉及 crate

### `proc_macro`

这是内置的库。过程宏与之交互的主要类型是 `proc_macro::TokenStream`，只能在 proc-macro crate 中使用。

### `proc_macro2`

它对 `proc_macro` 进行了包装，因此这两个库的所定义类型大多可以相互转化。

更重要的是，`proc_macro2` 被设计在 proc-macro crate 之外使用，从而在 
lib/bin 中可以使用类似于过程宏的一些功能（比如在非 
proc-macro crate 中对过程宏进行打印、调试、测试）。

### `quote`

它利用 `quote! { #(#value),* }` 插值方式，把输入的内容变成 `proc_macro2::TokenStream` 类型[^proc_macro2::TokenStream]。

此外，它提供 `format_ident!` 来方便生成 `proc_macro2::Ident`。

[^proc_macro2::TokenStream]: 它可以使用 `.into()` 转化成 `proc_macro::TokenStream` （过程宏的输出）。

### `syn`

`syn` 基于 `proc_macro2` 和 `quote`，用于解析 `proc_macro::TokenStream`（过程宏的输入）、 Rust
语法或者自定义语法到其内部所定义的语法树节点类型，以及构建和操作节点类型。

最终通过 `quote::ToToken` trait 把 `syn` 节点类型转化成 `quote!` 可插值的内容，从而返回给编译器。

## `Span` 的作用

> 过程宏使用 [`Span`] 
> 类型来把每个标记和源码的位置、卫生性信息联系起来。为了让编译器生成的错误在正确的地方显示出来，过程宏负责正确传播和操作这些 
> `Span`。
>
> [src]：*proc-macro-workshop*/seq/tests/08-ident-span.rs

即这些 `Span` 代表标记的源码位置，主要目的是在出现错误时，把错误信息定位到错误源头。

你可以浏览声明宏与不同 Span 的过程宏的错误信息：[案例#assert_sync](./study-case.html#assert_sync)。

当然，它最终是代表位置，因此你可以用它解析其他信息：比如 syn 不提供普通的 `//` 注释解析，但你可以利用位置手动去解析它们，见
[syn: Non-doc comments](https://github.com/dtolnay/syn/issues/946)。

[src]: https://github.com/dtolnay/proc-macro-workshop/blob/0e90cf2551e42f85620aca092b4255fa1bd10660/seq/tests/08-ident-span.rs
[`Span`]: https://docs.rs/proc-macro2/latest/proc_macro2/struct.Span.html

## 使用同一个 proc-crate 中的多个过程宏


```rust, ignored
// sorted crate: lib.rs
#[proc_macro_attribute]
pub fn sorted(arg: TokenStream, input: TokenStream) -> TokenStream { ... }

#[proc_macro_attribute]
pub fn check(arg: TokenStream, input: TokenStream) -> TokenStream  { ... }
```

```rust,ignore
// 方法一：引入再使用属性
use sorted::sorted;
use sorted::check;
#[sorted]
#[check]

// 方法二：直接使用属性
#[sorted::sorted]
#[sorted::check]
```

## proc-macro-workshop

[proc-macro-workshop](https://github.com/dtolnay/proc-macro-workshop) 是什么？

它是 dtolnay 提供的一份过程宏实战教程。你也可以把它看做带你深入了解编写过程宏的 rustlings。

建议：
- 按照作者介绍的顺序编写，因为这个顺序是由易到难的；
- 千万不要忽视 README 和代码中的所有注释；
- 千万不要忽视 syn 和 quote 的文档及其文档样例代码。

你可以参考我的解答（未参考别的解答，完全由我独立编写）：
[https://github.com/zjp-CN/proc-macro-workshop](https://github.com/zjp-CN/proc-macro-workshop)



---

其他参考资料：
1. [Procedural Macros: The Basics (via Jonas Platte)](https://blog.turbo.fish/proc-macro-basics/)
