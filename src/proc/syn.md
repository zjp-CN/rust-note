# syn

这里主要结合编写经验来总结 syn 的使用方式，少数内容是对 syn 文档的翻译和重新组织。

如果内容上有出入，请以 syn 文档为准。

## 整体介绍

> 文档：[https://docs.rs/syn](https://docs.rs/syn)

syn 主要是一个解析库，用于把 Rust 标记流解析为 Rust 源代码的语法树。目前该库面向过程宏，但包含一些可能更通用的 API。

作者自己从以下几个方面介绍 syn，并给出了典型的代码。

- 数据结构方面：syn 提供一个完整的、可以表示任何有效的 Rust 源代码的语法树。
    - `syn::File` 就是这棵语法树的根节点，这个类型表示一个完整的代码源文件；
    - 更常见的情况是使用其他入口，比如 `syn::Item`、`syn::Expr` 和 `syn::Type`，它们都是枚举体，对应于 Rust 中的语法概念。
    - 几乎所有语法树节点的文档都给出了典型例子或者简明说明，非常易于理解和使用。
- `#[derive]` 方面：为解析 derive 宏的标记流提供 `syn::DeriveInput` 类型。
- 解析方面：
    - [`Parse`] trait 提供对 `ParseStream` 类型使用 `parse` 方法来将其解析成实现了此 trait 的基础（或自定义）类型；
    - syn 提供的每种语法树节点类型都可以单独解析和多项重组，由此轻松构建起全新的自定义语法；
    - 更深入的解析参考 [syn::parse](https://docs.rs/syn/latest/syn/parse/index.html) 模块文档。
- 位置信息： syn 解析的每个标记都与一个 [`Span`]
  相关联，该类型用于跟踪源代码中的标记的行和列信息，从而让过程宏指定在源代码位置上显示错误消息。
- feature 控制：你只需要启用你所需要的功能，而不必开启不需要的功能。

[`Span`]: https://docs.rs/proc-macro2/*/proc_macro2/struct.Span.html

## 解析

> 文档：[https://docs.rs/syn/latest/syn/parse/index.html](https://docs.rs/syn/latest/syn/parse/index.html)

### `parse_macro_input!`

这个宏充当过程宏的解析入口：无论哪种过程宏，也无论转化成 syn 定义的语法树还是你自定义的语法树，几乎都从这个宏开始。

一个最基本的 derive 宏的完整例子是：

```rust,ignore
#// Cargo.toml 中写入以下内容：
#// [dependencies]
#// syn = "1.0"
#// quote = "1.0"
#// 
#// [lib]
#// proc-macro = true

use proc_macro::TokenStream;

#[proc_macro_derive(MyMacro)]
pub fn my_macro(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    TokenStream::new()
}
```

当你使用 `cargo expand --lib` 命令，可以看到如下结果（点击右上角的取消隐藏看到完整内容）：

```rust,ignore
##![feature(prelude_import)]
##[prelude_import]
#use std::prelude::rust_2021::*;
##[macro_use]
#extern crate std;

use proc_macro::TokenStream;
#[proc_macro_derive(MyMacro)]
pub fn my_macro(input: TokenStream) -> TokenStream {
    let input = match ::syn::parse_macro_input::parse::<syn::DeriveInput>(input) {
        ::syn::__private::Ok(data) => data,
        ::syn::__private::Err(err) => {
            return ::syn::__private::TokenStream::from(err.to_compile_error());
        }
    };
    TokenStream::new()
}

#const _: () = {
#    extern crate proc_macro;
#    #[rustc_proc_macro_decls]
#    #[allow(deprecated)]
#    static _DECLS: &[proc_macro::bridge::client::ProcMacro] =
#        &[proc_macro::bridge::client::ProcMacro::custom_derive(
#            "MyMacro",
#            &[],
#            my_macro,
#        )];
#};
```

尽管以下内容不完全与 `parse_macro_input!` 有关，但是为了深入这段展开的代码，你需要确切地掌握以下内容，并且注意一些事情。

#### 卫生性

涉及 *prelude* 的几行，以及 `#[macro_use] extern crate std;` 是每个 rs
文件都会自动添加的内容，它们用于导入最常用的 items，和 std 的[所有公开的声明宏][std-macros]。由此，你可以直接使用
`Option`、`Result`（以及它们的成员）这些数据结构与 traits，而不必使用绝对路径或者手动 use。而
`println!` 这类常见的宏，以及你可能不会经常使用的宏（比如 [`compile_error!`]），也会被加载进来。

了解这一点很重要，因为过程宏[不是卫生的][unhygienic]：
时刻警醒自己，不要轻易假设使用者的句法上下文，以免让意外的重名标识符污染代码，尽可能在 **生成的标记中** 使用
**[绝对路径]**、或者添加醒目的标识前后缀 —— 使用 `::std::result::Result` 而不是 
`Result`，对生成的 **内部函数名或内部变量名** 使用 `__internal_foo` 而不是 `foo`。

[绝对路径]: https://doc.rust-lang.org/nightly/reference/paths.html#path-qualifiers

#### 宏展开

[`parse_macro_input!`] 被展开成以下 match 语句：
```rust,ignore
let input = match ::syn::parse_macro_input::parse::<syn::DeriveInput>(input) {
    ::syn::__private::Ok(data) => data,
    ::syn::__private::Err(err) => {
        return ::syn::__private::TokenStream::from(err.to_compile_error());
    }
};
```
它是个声明宏，所做的事情很简单，解析成功则取出数据，解析失败则直接返回错误。

它有两种语法：
- `($tokenstream:ident as $ty:ty)` 或者 `($tokenstream:ident)`：即把实现 [`Parse`] trait 
    的类型放到宏的语法内，或者放到模式语法上
```rust,ignore
let input = syn::parse_macro_input!(input as syn::DeriveInput);
let input: syn::DeriveInput = syn::parse_macro_input!(input); // 或者在 binding 时指明类型
```
- `($tokenstream:ident with $parser:path)`：使用具有 [`Parser`] trait 的类型，这个 trait 专门为函数而设计，
    函数签名满足 `FnOnce(ParseStream) -> Result<T>` 即可。

此外，你肯定会好奇这里的 [`syn::__private`][__private] 模块，它没什么特别的，里面是
std、quote、proc_macro2、proc_macro 和 syn 其他模块的 reexport，这印证了第一点的建议：在宏中使用绝对路径。


你还需要观察到以下三点：
- `::syn::parse_macro_input::parse` 这个内部函数与 [`syn::parse`] 函数几乎别无二致，都把
    `proc_macro::TokenStream` 转化成 实现了 [`Parse`] trait 的类型。但它们的唯一的区别在于，
    `::syn::parse_macro_input::parse` 增加支持解析 [`AttributeArgs`]，此类型用在解析属性宏参数上。
- `Err(err)` 中的 `err` 是 [`syn::parse::Error`] 类型，也是 [`syn::Error`][`Error`] 类型。因为这两个 
    `Error` 类型都是从 `syn::error` 私有模块的重导出。
    [`syn::Result`][`Result`] 和 [`syn::parse::Result`] 也是同一类型，因为它们是类型别名 + 重导出。
- 这个宏使用的另一个限制条件：必须用在返回值为 `proc_macro::TokenStream` 类型的函数中。

[`parse_macro_input!`]: https://docs.rs/syn/latest/syn/macro.parse_macro_input.html
[`Parser`]: https://docs.rs/syn/latest/syn/parse/trait.Parser.html
[std-macros]: https://doc.rust-lang.org/std/index.html#macros
[`compile_error!`]: https://doc.rust-lang.org/std/macro.compile_error.html
[unhygienic]: https://doc.rust-lang.org/nightly/reference/procedural-macros.html#procedural-macro-hygiene

[__private]: https://github.com/dtolnay/syn/blob/master/src/export.rs
[`Parse`]: https://docs.rs/syn/latest/syn/parse/trait.Parse.html
[`syn::parse::Error`]: https://docs.rs/syn/latest/syn/parse/struct.Error.html
[`Error`]: https://docs.rs/syn/latest/syn/struct.Error.html
[`Result`]: https://docs.rs/syn/latest/syn/type.Result.html
[`syn::parse::Result`]: https://docs.rs/syn/latest/syn/parse/type.Result.html
[`syn::parse`]: https://docs.rs/syn/latest/syn/fn.parse.html
[`AttributeArgs`]: https://docs.rs/syn/latest/syn/type.AttributeArgs.html

#### `const _` 技巧

最后的 `const _` 部分是过程宏库自动添加的，它用于[初始化][ProcMacro::custom_derive]过程宏。

这是一种不起眼的语法，你很少真正手写它，但它却是宏非常常用的技巧。它有正式的名称 —— 
[unnamed const][unnamed-constant]，具有很好的性质：可以重复定义。

无论是过程宏还是声明宏，你都会感受到这种写法给你提供便利：在只能定义 item 
的地方，它给你一个编译期求值的局部作用域，在其中你可以给某个类型实现 trait，而且可以定义不影响源代码的临时
const 或 type alias 或数据结构。

在 [proc-macro-workshop] 的最后一个案例 [bitfield] 中，你势必会使用这种技巧。比如[这样][bitfield-spc]或者[这样][bitfield-bit]。

[ProcMacro::custom_derive]: https://github.com/rust-lang/rust/blob/0bcacb391b28460f5a50fd627f01f670dfcfc7cc/library/proc_macro/src/bridge/client.rs#L462
[unnamed-constant]: https://doc.rust-lang.org/nightly/reference/items/constant-items.html#unnamed-constant
[proc-macro-workshop]: https://github.com/dtolnay/proc-macro-workshop
[bitfield]: https://github.com/dtolnay/proc-macro-workshop#attribute-macro-bitfield
[bitfield-spc]: https://github.com/zjp-CN/proc-macro-workshop/blob/d063faae6b622d146b6d156a514bce976ed7bf40/bitfield/impl/src/spe.rs#L38-L63
[bitfield-bit]: https://github.com/zjp-CN/proc-macro-workshop/blob/d063faae6b622d146b6d156a514bce976ed7bf40/bitfield/impl/src/bit.rs#L53-L79

### `Parse` trait

[`Parse`] 是 syn 里对外用途最广泛的 trait （之一，另一个对外被广泛使用的 trait 是 [`quote::ToTokens`]）。

[`syn::ItemStruct`]: https://docs.rs/syn/latest/syn/struct.ItemStruct.html#impl-Parse
[`quote::ToTokens`]: https://docs.rs/quote/*/quote/trait.ToTokens.html

它很简单，因为只有一个方法：

```rust,ignore
pub trait Parse: Sized {
    fn parse(input: ParseStream<'_>) -> Result<Self>;
}
```

syn 根模块中（`syn::*`）的大多数结构体和枚举体都实现了此 trait，其功能就是把语法标记的缓冲流解析成语法树。

一个典型的使用方法如下：

```rust,ignore
#// src: https://docs.rs/syn/latest/syn/parse/index.html#example
#use syn::{
#    braced,
#    parse::{Parse, ParseStream},
#    punctuated::Punctuated,
#    token, Field, Ident, Result, Token,
#};
struct ItemStruct {
    struct_token: Token![struct],
    ident: Ident,
    brace_token: token::Brace,
    fields: Punctuated<Field, Token![,]>,
}

impl Parse for ItemStruct {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(ItemStruct {
            struct_token: input.parse()?,
            ident: input.parse()?,
            brace_token: braced!(content in input),
            fields: content.parse_terminated(Field::parse_named)?,
        })
    }
}
```

这是 [`syn::ItemStruct`] 实现 `Parse` 的一个简要版本，你只需要使用 `input.parse()?`
就能解析一个语法标记。这得益于 Rust 的泛型和 syn 给你写好的基础解析类型及其 `Parse` 实现。

这段样例代码不难看懂，但里面的细节得琢磨琢磨。

#### `Token!` 和 `braced!`

这里涉及两个宏：`Token!` 和 `braced!`，其中 `braced!` 与 `bracketed!` 和 `parenthesized!` 用法一致。

使用 `cargo expand` 展开得到：

```rust,ignore
struct ItemStruct {
    struct_token: ::syn::token::Struct,
    ident: Ident,
    brace_token: token::Brace,
    fields: Punctuated<Field, ::syn::token::Comma>,
}

impl Parse for ItemStruct {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(ItemStruct {
            struct_token: input.parse()?,
            ident: input.parse()?,
            brace_token: match ::syn::group::parse_braces(&input) {
                ::syn::__private::Ok(braces) => {
                    content = braces.content;
                    braces.token
                }
                ::syn::__private::Err(error) => {
                    return ::syn::__private::Err(error);
                }
            },
            fields: content.parse_terminated(Field::parse_named)?,
        })
    }
}
```

- [`syn::Token!`] 声明宏做的事情非常简单，把标点或者关键字直接替换成 [`syn::token`] 模块下对应的解析类型：
    把 `Token![struct]` 替换成 `::syn::token::Struct`，把 `Token![,]` 替换成 
    `::syn::token::Comma`。任何需要写出 `syn::token::xx` 的地方，都可以使用这个宏，而无需记住其类型名称。\
    作者为了展示这个功能“特别”的宏，特意使用了大写，以及方括号 `[]`。
- [`syn::braced!`] 声明宏与 `syn::parse_macro_input!` [类似](./syn.md#宏展开)，被展开成 match 语句来处理
    `Result`。但它需要一个预先声明的 `content` 变量，当 `content` 经过 `braced!` 处理之后，为 
    `ParseStream` 类型，用于解析花括号 `{}` 内部的标记（如这里的 `fields`）。\
    而 `bracketed!` 和 `parenthesized!` 也是一样的用法，区别在于后两者用在解析方括号 `[]` 和圆括号 `()` 上。

[`syn::Token!`]: https://docs.rs/syn/latest/syn/macro.Token.html
[`syn::token`]: https://docs.rs/syn/latest/syn/token/index.html
[`syn::braced!`]: https://docs.rs/syn/latest/syn/macro.braced.html

接下来，把目光放到主角 `ParseStream` 类型上。

#### `ParseBuffer` 与 `ParseStream`

[`ParseStream`] 是 [`ParseBuffer`] 的共享引用的类型别名：

```rust,ignore
type ParseStream<'a> = &'a ParseBuffer<'a>;
```

我们总是在 syn 的解析函数的函数签名中把 `ParseStream` 
作为函数参数，而在解析函数内部，调用其方法时，应该参考 [`ParseBuffer`] 的文档。

此外，`ParseStream` 是共享引用，它实现了 [`Copy`](https://doc.rust-lang.org/std/marker/trait.Copy.html#impl-Copy-73) 
trait：任何使用它的地方，都是使用引用的复制品，而不具有移动语义。

[`ParseBuffer`] 表示缓冲标记流的游标位置，其背后的核心类型是 [`Cursor`] （后面会谈到这个类型）。

你无法在 syn 之外的代码构造 `ParseBuffer`，只通过三条公开的解析入口接触到此类型：
1. [`parse_macro_input!`] 用于解析过程宏的输入；
2. `syn::parse*` 函数用于中途解析语法；
3. [`Parser`] trait 用于抽象所有解析函数。

它的文档非常友好，这里只概括地浏览一遍：

| 位置 | 方法名                     | 返回值类型[^T-P-R]           | 说明                                                                                                                                                          |
|------|----------------------------|------------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------|
| 推进 | parse(&self)               | Result\<T\>                  | 成功解析当前游标下的语法标记之后，把游标位置推进到下一个标记                                                                                                  |
| 推进 | call(&self, f)             | Result\<T\>                  | 与 parse 方法类似，但使用一个解析函数 f                                                                                                                       |
| 不变 | peek(&self, t)             | bool                         | 判断下一个标记是否是 t                                                                                                                                        |
| 不变 | peek2(&self, t)            | bool                         | 判断下下一个标记是否是 t                                                                                                                                      |
| 不变 | peek3(&self, t)            | bool                         | 判断下下下一个标记是否是 t                                                                                                                                    |
| 不变 | lookahead1(&self)          | Lookahead1                   | 可调用 `.peek()` 判断下一个标记是否是一组标记中的某一个，<br>如果不是，可调用 `.error()` 方法来返回解析错误                                                   |
| 不变 | fork(&self)                | Self                         | 不建议使用；其实质是将 Self 复制（游标的复制成本很低），但需要注意解析成本；<br>此方法可搭配 [`Speculative`] trait                                            |
| 不变 | is_empty(&self)            | bool                         | 判断解析流是否还有待解析的标记                                                                                                                                |
| 不变 | span(&self)                | Span                         | 如果解析流导了末尾（无待解析的标记），返回 `Span::call_site()` （过程宏在源码的位置）；<br>否则返回当前游标下（即下一个标记）的 Span                          |
| 不变 | error(&self, message)      | syn::Error                   | 在当前游标位置上返回错误信息                                                                                                                                  |
| 不变 | cursor(&self)              | Cursor                       | 底层的解析 API；复制当前游标位置，对其返回的游标做任何操作都不会影响解析流的游标                                                                              |
| 结束 | parse_terminated(&self, f) | Result\<Punctuated\<T, P\>\> | 从当前游标解析直到解析流的末尾，解析成 0 次或多次 T，每个 T 的分隔符为 P，<br>且最后的分隔符可以出现也可以不出现                                              |
| 推进 | step(&self, f)             | Result\<R\>                  | 强大但底层的解析 API，但在 syn 之外很少使用（因为你很少直接使用 `Cursor`）：<br>f 类似于 `fn(Cursor) -> Result<(R, Cursor)>` 形式即可；解析成功将自动推进游标 |

[^T-P-R]: T 和 P 都为实现了 Parse 的类型，即 `T: Parse`、`P: Parse`；`R` 可为任意类型。

你可以观察到，这些方法都只有 `&self`，而没有 `&mut self`，而推进游标理应需要独占引用。这是因为 `ParseBuffer`
采用了“内部可变性”设计，其背后使用了 [`Cell`]。

[`ParseBuffer`]: https://docs.rs/syn/latest/syn/parse/struct.ParseBuffer.html
[`ParseStream`]: https://docs.rs/syn/latest/syn/parse/type.ParseStream.html
[`Cursor`]: https://docs.rs/syn/latest/syn/buffer/struct.Cursor.html
[`Speculative`]: https://docs.rs/syn/latest/syn/parse/discouraged/trait.Speculative.html
[`Cell`]: https://doc.rust-lang.org/std/cell/struct.Cell.html

### `Parser` trait

[`Parser`] 是 syn 的解析函数抽象，功能是把标记流转化成语法树节点。

对 syn 的使用者来说，它可以用在 `parse_macro_input!`
上，比如文档给的[例子](https://docs.rs/syn/latest/syn/macro.parse_macro_input.html#usage-with-parser)。

但除此之外，你很少直接使用它的方法，也很少关注哪些函数实现了 `Parser` trait。比如 
[`Parse`] trait 的唯一方法就[实现了][Parse-Parser] `Parser` trait，但你
[不必写](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=f3476f9caab9111bd35d1d59328ccde5)
`ItemStruct::parse.parse2(token_stream)`。

虽然 `Parser` 就是对 `TokenStream -> T` 的抽象（`TokenStream` 来自于过程宏的函数输入参数），而以下实现

```rust,ignore
impl<F, T> Parser for F
  where F: FnOnce(ParseStream<'_>) -> Result<T> 
```

把这种数据类型的变换转化成 `ParseStream -> T`，从而 syn 的大部分解析方式都只针对和基于 `ParseStream` （即
[`ParseBuffer`](./syn.html#parsebuffer-与-parsestream)）。

以至于，即使以下函数实现了 `Parser` trait，你也几乎只在给某类型实现 `Parse` trait 时使用它们[^parse-parse_terminated]：

- syn::punctuated::Punctuated::parse_terminated
- syn::punctuated::Punctuated::parse_separated_nonempty
- syn::Attribute::parse_outer
- syn::Attribute::parse_inner
- syn::Field::parse_named
- syn::Field::parse_unnamed
- syn::Block::parse_within

[^parse-parse_terminated]: 比如[上述例子](./syn.html#parse-trait)的 `content.parse_terminated()`

---

你可能会发现 syn 的文档中有直接使用 `Parser` trait 
方法的[例子](https://docs.rs/syn/latest/syn/parse/index.html#the-parser-trait)。

那的确是一种写法，但缺点也显而易见：`Parser` 所定义的方法需要消耗 `TokenStream` 的所有权。

这意味着每次解析一部分标记，你需要复制一次标记流（复制 `TokenStream` 的成本比较高，所以才会有 
`ParseBuffer` —— 或者说 [`syn::buffer`]）。

总而言之，`Parse` 比 `Parser` 更面向使用者，它们代表不同的抽象。

[`syn::buffer`]: https://docs.rs/syn/latest/syn/buffer/index.html
[Parse-Parser]: https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=54d21dd08bdc98edf4cfc93c9885dfc2

### `syn::parse*` 函数

基于 `ParseStream` 并不是 syn 唯一的解析途径。syn 提供以下解析函数（重点在函数参数上）：

| 函数                                                               | 说明                                         |
|--------------------------------------------------------------------|----------------------------------------------|
| [`fn parse<T: Parse>(tokens: TokenStream) -> Result<T>`][`parse`]  | 从 `proc_macro::TokenStream` 中直接解析成 T  |
| [`fn parse2<T: Parse>(tokens: TokenStream) -> Result<T>`][`parse`] | 从 `proc_macro2::TokenStream` 中直接解析成 T |
| [`fn parse_str<T: Parse>(s: &str) -> Result<T>`][`parse_str`]      | 从字符串中解析成类型 T                       |
| [`fn parse_file(content: &str) -> Result<File>`][`parse_file`]     | 从字符串中解析成类型 [`File`]                |

[`parse`]: https://docs.rs/syn/latest/syn/fn.parse.html
[`parse`]: https://docs.rs/syn/latest/syn/fn.parse2.html
[`parse_str`]: https://docs.rs/syn/latest/syn/fn.parse_str.html
[`parse_file`]: https://docs.rs/syn/latest/syn/fn.parse_file.html
[`File`]: https://docs.rs/syn/latest/syn/struct.File.html

它们背后都涉及 `Parser` trait，可以看做 `Parser` trait 的用户级接口。
`parse` 和 `parse2` 的区别在于 `TokenStream` 的来源库不同，通常我们使用 `2` 后缀区分来自 `proc_macro2` 
的内容，尤其是以下写法：

```rust,ignore
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
```

`parse` 和 `parse_macro_input!` 
的区别已经在[宏展开](./syn.md#宏展开)部分说过了，后者可以解析属性宏的参数  [`AttributeArgs`]：

```rust,ignore
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn my_attribute(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(args as syn::AttributeArgs);
    /* ... */
}
```

### `parse_quote!` 与 `parse_quote_spanned!`

回顾一下 [`quote::quote!`][`quote!`] 的功能：
- 它是一个声明宏，其展开的结果是 `proc_macro2::TokenStream` 类型；
- 它有自己的插值语法 `#( #val ),*`，其中 `,` 是可选的任意分隔符，且 `val` 满足以下一个条件即可：
    - val 实现了 `Iterator` trait，且 `Iterator::Item` 实现了 [`ToTokens`][`quote::ToTokens`] trait
    - val 是 `Vec` 或 `slice` 或 `BTreeSet`，且其元素实现了 [`ToTokens`][`quote::ToTokens`] trait

而 [`quote::quote_spanned!`] 比 `quote!` 增加了 `span=>` 语法，即给生成的标记附带自定义的 [`Span`]。

[`quote!`]: https://docs.rs/quote/*/quote/macro.quote.html
[`quote::quote_spanned!`]: https://docs.rs/quote/*/quote/macro.quote_spanned.html

[`parse_quote!`] 与 `quote!` 的不同之处在于，`parse_quote!` 的展开结果为 `T`，`T` 满足以下一种情况：
- `T` 实现了 [`Parse`] trait
- `T` 是 `Attribute` 或 `Punctuated<T, P>` 或 `Vec<Stmt>`

它的名字听起来有点奇怪，但你看它的实现，就非常容易理解什么叫做 `parse + quote!`：

```rust,ignore
#// src: https://docs.rs/syn/latest/src/syn/parse_quote.rs.html#70-74
macro_rules! parse_quote {
    ($($tt:tt)*) => {
        $crate::parse_quote::parse($crate::__private::quote::quote!($($tt)*))
    };
}

// $crate::parse_quote::parse 函数定义如下
// Not public API.
#[doc(hidden)]
pub fn parse<T: ParseQuote>(token_stream: proc_macro2::TokenStream) -> T {
    let parser = T::parse;
    match parser.parse2(token_stream) {
        Ok(t) => t,
        Err(err) => panic!("{}", err),
    }
}

// Not public API.
#[doc(hidden)]
pub trait ParseQuote: Sized {
    fn parse(input: ParseStream) -> Result<Self>;
}

impl<T: Parse> ParseQuote for T {
    fn parse(input: ParseStream) -> Result<Self> {
        <T as Parse>::parse(input)
    }
}

impl ParseQuote for Attribute { /* 省略 */}
impl<T: Parse, P: Parse> ParseQuote for Punctuated<T, P> { /* 省略 */ }
impl ParseQuote for Vec<Stmt> { /* 省略 */ }
```

理解了 `parse_quote!` 与 `quote!` 之间的区别，那么就能理解 `parse_quote_spanned!` 与
`quote_spanned!` 的区别：它们只在返回类型上不同。

但，这有什么意义呢？

大部分情况下你只需关注和使用 `quote!`，因为它的目的是生成语法标记，也是生成 `proc_macro::TokenStream`
的最常见方式。（忽略 `proc_macro::TokenStream` 与 `proc_macro2::TokenStream` 之间的差异，它们之间的转化只需要 `from-into` ）

有些情况下，当你真正需要构造 syn 的某种类型时，尽管那种类型的字段都是公开的，但你很少直接用结构体语法构造它们。

假设你想构造闭包表达式  `|| a + b`，它的直接类型是 [`ExprClosure`]，你的函数签名需要
[`Expr`]，它们之间转化只需要 `from-into`，但你如何得到 `ExprClosure`？

它有 9 个字段，而且 `Path`、`Punctuated` 类型手动构造起来有些繁琐。

有了 `parse_quote!`，你只需要 `let d: syn::Expr = parse_quote! { || a + b };` 一行语句即可。

它完整的[展现][parse_quote-ex]如下（点击右上角的取消隐藏看到完整内容）：

```rust,ignore
Expr::Closure(
#    ExprClosure {
#        attrs: [],
#        asyncness: None,
#        movability: None,
#        capture: None,
#        or1_token: Or,
#        inputs: [],
#        or2_token: Or,
#        output: Default,
#        body: Binary(
#            ExprBinary {
#                attrs: [],
#                left: Path(
#                    ExprPath {
#                        attrs: [],
#                        qself: None,
#                        path: Path {
#                            leading_colon: None,
#                            segments: [
#                                PathSegment {
#                                    ident: Ident {
#                                        sym: a,
#                                    },
#                                    arguments: None,
#                                },
#                            ],
#                        },
#                    },
#                ),
#                op: Add(
#                    Add,
#                ),
#                right: Path(
#                    ExprPath {
#                        attrs: [],
#                        qself: None,
#                        path: Path {
#                            leading_colon: None,
#                            segments: [
#                                PathSegment {
#                                    ident: Ident {
#                                        sym: b,
#                                    },
#                                    arguments: None,
#                                },
#                            ],
#                        },
#                    },
#                ),
#            },
#        ),
#    },
)
```

在我没有真正搞清楚这几个宏的区别之前，我写了这个例子 [struct_new]，为了做同样一件事，它有两个版本：完全不使用
`parse_quote!` 和尽可能使用 `parse_quote!`。

然而，真正去构建具体的解析类型是必要的吗？

你的函数可能只需要（或者返回） `proc_macro2::TokenStream` 类型就好了，不必把这些标记转来转去：
`parse_quote!` 只不过是把语法用 `quote!` 转成 `proc_macro2::TokenStream`，然后用 `Parser` 再转成你要的类型。

请相信 `quote!` 足够聪明，产生你想要的语法标记。别忘了它的特点 “quasi-quoting”。

[`Expr`]: https://docs.rs/syn/latest/syn/enum.Expr.html
[`ExprClosure`]: https://docs.rs/syn/latest/syn/struct.ExprClosure.html
[parse_quote-ex]: https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=8f8b9504d786e02381f77ea684a3e429
[`parse_quote!`]: https://docs.rs/syn/latest/syn/macro.parse_quote.html
[`parse_quote_spanned!`]: https://docs.rs/syn/latest/syn/macro.parse_quote_spanned.html
[struct_new]: https://github.com/zjp-CN/structs_new

## buffer

[`syn::buffer`] 模块只有两个结构体：
- [`TokenBuffer`] 高效多次遍历标记流的缓冲标记流，只有三个公有方法：
    - `fn new(stream: proc_macro::TokenStream) -> Self` 构造缓冲
    - `fn new2(stream: proc_macro2::TokenStream) -> Self` 构造缓冲
    - `fn begin(&self) -> Cursor<'_>` 从缓冲的第一个标记位置上产生游标，我们就是利用这个游标遍历缓冲标记流
- [`Cursor`] 高效复制的游标，是不可变数据（缓冲标记流）的共享引用，其背后是裸指针。
    - 它的亮点是实现了 `Copy` trait，，这意味着你可以隐式复制来驻足在这个游标上。
    - 除 `empty` 方法创建空游标之外，其余所有方法都是 `fn xx(self) -> ..` 形式。且大部分方法是
      `fn xx(self) -> Option<(.., Self)>` 形式，这意味着每次调用其方法，都是把这个游标消耗掉，然后得到下一个标记的游标。
    - 两个游标可以比较相等：当它们在同一个标记流中位置相同，且 Span 相同时，两个游标相等。
    - 它属于低层级 API，其大部分方法与 `proc_macro2` 的数据结构有关，所以你想使用它，得掌握 [`TokenTree`]。

通常，你很少关注到这两个类型，但它们在 syn 中作为底层数据结构被使用。

一个很好的案例是 proc-macro-workshop 的
`Seq`，你可以参考我的[解答](https://github.com/zjp-CN/proc-macro-workshop/blob/master/seq/src/lib.rs)。


[`TokenBuffer`]: https://docs.rs/syn/latest/syn/buffer/struct.TokenBuffer.html
[`TokenTree`]: https://docs.rs/proc-macro2/*/proc_macro2/enum.TokenTree.html

## `visit` | `visit_mut` | `fold`

这三个模块都是在遍历语法树，且默认以递归形式遍历，它们之间的区别与所有权概念对应：

```rust,ignore
// 以共享引用方式遍历语法树节点
pub trait Visit<'ast> {
    fn visit_expr_binary(&mut self, node: &'ast ExprBinary) {
        visit_expr_binary(self, node);
    }

    /* ... */
}

// 以独占引用方式遍历语法树节点
pub trait VisitMut {
    fn visit_expr_binary_mut(&mut self, node: &mut ExprBinary) {
        visit_expr_binary_mut(self, node);
    }

    /* ... */
}

// 以所有权方式遍历语法树节点
pub trait Fold {
    fn fold_expr_binary(&mut self, node: ExprBinary) -> ExprBinary {
        fold_expr_binary(self, node)
    }

    /* ... */
}
```

[`visit`] 的方式可以在遍历语法树时，把某种节点类型的引用都提取出来； [`visit_mut`] 可以在遍历时修改节点类型；而
[`fold`] 可以遍历时消耗某种节点类型来生成这种节点类型。

它们的文档都有最简代码样例，很好理解。其他案例可参考
- syn example: [trace-var](https://github.com/dtolnay/syn/blob/master/examples/trace-var/trace-var/src/lib.rs)
- proc-macro-workshop: [sorted](https://github.com/zjp-CN/proc-macro-workshop/blob/master/sorted/src/_fn.rs)
- [you_can::turn_off_the_borrow_checker](https://docs.rs/you-can/latest/you_can/attr.turn_off_the_borrow_checker.html)

[`visit`]: https://docs.rs/syn/latest/syn/visit/index.html
[`visit_mut`]: https://docs.rs/syn/latest/syn/visit_mut/index.html
[`fold`]: https://docs.rs/syn/latest/syn/fold/index.html

