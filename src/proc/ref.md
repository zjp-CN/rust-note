# 基础内容

> 此部分内容翻译和整理自：[reference#procedural-macros](https://doc.rust-lang.org/nightly/reference/procedural-macros.html)

## 过程宏是什么

过程宏必须定义在 `proc-macro` 类型的库：

```toml
[lib]
proc-macro = true
```

使用过程宏的目的是：在编译期间操作 Rust 的语法，包括消耗和生成 Rust 语法。

过程宏被视为从一个 AST 到另一个 AST 的函数。

像函数一样，过程宏：

1. 要么返回语法：替换或者删除语法
2. 要么导致 panic：编译器捕获这些 panic，并将其转化为编译器的错误信息
3. 要么一直循环：让编译器处于挂起状态 (hanged)


过程宏享受与编译器相同的资源，可以：

1. 访问编译器才能访问的标准输入/输出/错误 ；
2. 可以访问编译期间生成的文件（如同 [`build.rs`](https://doc.rust-lang.org/nightly/cargo/reference/build-scripts.html) 一样）。


过程宏报告错误的方式：

1. panic
2. 调用 [`compile_error`](https://doc.rust-lang.org/nightly/std/macro.compile_error.html)

## `proc_macro` crate

[`proc_macro`](https://doc.rust-lang.org/nightly/proc_macro/index.html) 是 Rust 编译器所提供的，提供了编写过程宏所需的类型和工具。

1. [`TokenStream`](https://doc.rust-lang.org/nightly/proc_macro/struct.TokenStream.html)
    类型是一系列标记，用于迭代 token trees 或者从 token trees 聚集到一起。它几乎等价于 
    `Vec<TokenTree>`（但它的 clone 是低成本的），其中 
    [`TokenTree`](https://doc.rust-lang.org/nightly/proc_macro/enum.TokenTree.html)
    几乎可以被视为词法标记 (lexical token)。
2. [`Span`](https://doc.rust-lang.org/proc_macro/struct.Span.html)
    表示源代码的范围，而且它主要用于报告错误。所有的标记都伴随着一个 `Span`，你不能修改 
    `Span`，但是可以生成 `Span`。

## 卫生性

过程宏不是卫生的 (unhygienic)，这意味着：
1. 生成的 token stream 内联写入到紧邻的代码
2. 生成的 token stream 受到外部引入 items 的影响

所以要时刻警惕这一点。例如
- 应该使用绝对路径，而不应该依赖预先引入的 items：即使用 `::std::option::Option` 而不应该使用
  `Option`，因为**有可能**在上下文存在同名的 `Option`，从而造成标识符混乱。
- 应该确保生成的函数不与其他函数同名：比如在生成的函数名中使用 `__internal_foo`，而不使用 `foo`。

## 三类过程宏

### 函数式

类似于函数调用，但使用宏调用符号 `!` 的宏。

函数式过程宏的定义方式：
1. 使用公有函数 `pub fn` 声明，函数名即宏名
2. 函数带有 `#[proc_macro]` 属性
3. 函数签名为 `(TokenStream) -> TokenStream`

MWE 如下：

```shell
$ cargo new proc
$ cd proc
$ # 修改 lib 的类型
$ echo "[lib]
proc-macro = true" >> Cargo.toml
```

```rust,ignore
//! src/lib.rs
// extern crate proc_macro; // 2018 版本之后不需要这行代码
use proc_macro::TokenStream;

/// 函数式：`make_answer!()`
#[proc_macro]
pub fn make_answer(_item: TokenStream) -> TokenStream {
    "fn answer() -> u32 { 42 }".parse().unwrap()
}
```

```rust,ignore
//! src/main.rs
// extern crate proc; // 2018 版本之后不需要这行代码
use proc::make_answer;

make_answer!();

fn main() {
    println!("{}", answer()); 
}
```

```shell
$ cargo run # 将打印 42
```

注意：从 2018 版本之后（Rust 1.31+），引入外部库无需再写 `extern crate`，引入宏如引入 item 一样简洁，参考 
[The Edition Guide#Path and module system changes](https://doc.rust-lang.org/stable/edition-guide/rust-2018/path-changes.html)。

### `derive` 式

定义方式：
1. 使用公有函数 `pub fn` 声明
2. 函数带有 `#[proc_macro_derive(Name)]` 属性或者 `#[proc_macro_derive(Name, attributes(attr))]`
3. 函数签名为 `(TokenStream) -> TokenStream`

MWE（接着上面的例子）：
```rust,ignore
//! src/lib.rs

// 上面例子的函数式过程宏代码不变，追加以下内容

/// `derive` 式：`#[Derive(AnswerFn)]`
#[proc_macro_derive(AnswerFn)]
pub fn derive_answer_fn(_item: TokenStream) -> TokenStream {
    "fn answer_derive() -> u32 { 42 }".parse().unwrap()
}

/// `derive` 式：`#[Derive(AnswerFn)]` + `#[helper]`
#[proc_macro_derive(HelperAttr, attributes(helper))]
pub fn derive_helper_attr(_item: TokenStream) -> TokenStream { TokenStream::new() }
```

```rust,ignore
//! src/main.rs 以下内容覆盖上面的例子

#![allow(unused)]

// extern crate proc; // 2018 版本之后不需要这行代码
use proc::*;

make_answer!();

#[derive(AnswerFn)]
struct A;

#[derive(HelperAttr)]
struct B {
    #[helper]
    b: (),
}

#[derive(HelperAttr)]
struct C(#[helper] ());

fn main() {
    println!("{}", answer());
    println!("{}", answer_derive());
}
```

### 属性式

属性宏是附加到 [items](https://doc.rust-lang.org/nightly/reference/items.html) 的
[属性](https://doc.rust-lang.org/nightly/reference/attributes.html)。

定义方式：
1. 使用公有函数 `pub fn` 声明，函数名为属性名
2. 函数带有 `#[proc_macro_attribute]` 属性
3. 函数签名为 `(TokenStream, TokenStream) -> TokenStream`。第一个参数表示属性名之后的（括号分隔的）标记树，
   如果属性名之后无标记，第一个参数则为空。第二个参数表示该属性之后的内容（包括 items 和其他属性）。
   返回值表示用于替换原 item 的 item （可以任意多）。

MWE（接着上面的例子）：

```rust,ignore
//! src/lib.rs

// 函数式、derive 式代码不变，追加以下内容

/// 属性式：`#[show_streams]` 或者 `#[show_streams(attr)]`
#[proc_macro_attribute]
pub fn show_streams(attr: TokenStream, item: TokenStream) -> TokenStream {
    println!("attr: \"{}\"", attr.to_string());
    println!("item: \"{}\"", item.to_string());
    item
}
```

```rust,ignore
//! src/main.rs

// 追加以下内容

// Example: Basic function
#[show_streams]
fn invoke1() {}
// out: attr: ""
// out: item: "fn invoke1() { }"

// Example: Attribute with input
#[show_streams(bar)]
fn invoke2() {}
// out: attr: "bar"
// out: item: "fn invoke2() {}"

// Example: Multple tokens in the input
#[show_streams(multiple => tokens)]
fn invoke3() {}
// out: attr: "multiple => tokens"
// out: item: "fn invoke3() {}"

// Example:
#[show_streams { delimiters }]
fn invoke4() {}
// out: attr: "delimiters"
// out: item: "fn invoke4() {}"
```



### 完整 MEW

注意：`*.rs` 代码已隐藏，直接点击代码块右上角 `Copy to clipboard` 即可复制。

```toml
# Cargo.toml 内容
[package]
name = "proc"
version = "0.0.1"
edition = "2021"

[lib]
proc-macro = true
```

```rust,ignore
//! 复制以下内容到 src/lib.rs 文件
#//! src: https://doc.rust-lang.org/nightly/reference/procedural-macros.html
#// extern crate proc_macro; // 2018 版本之后不需要这行代码
#use proc_macro::TokenStream;
#
#/// 函数式：`make_answer!()`
##[proc_macro]
#pub fn make_answer(_item: TokenStream) -> TokenStream {
#    "fn answer() -> u32 { 42 }".parse().unwrap()
#}
#
#/// `derive` 式：`#[Derive(AnswerFn)]`
##[proc_macro_derive(AnswerFn)]
#pub fn derive_answer_fn(_item: TokenStream) -> TokenStream {
#    "fn answer_derive() -> u32 { 42 }".parse().unwrap()
#}
#
#/// `derive` 式：`#[Derive(AnswerFn)]` + `#[helper]`
##[proc_macro_derive(HelperAttr, attributes(helper))]
#pub fn derive_helper_attr(_item: TokenStream) -> TokenStream { TokenStream::new() }
#
#/// 属性式：`#[show_streams]` 或者 `#[show_streams(attr)]`
##[proc_macro_attribute]
#pub fn show_streams(attr: TokenStream, item: TokenStream) -> TokenStream {
#    println!("attr: \"{}\"", attr.to_string());
#    println!("item: \"{}\"", item.to_string());
#    item
#}
```

```rust,ignore
//! 复制以下内容到 src/main.rs 文件
#//! src: https://doc.rust-lang.org/nightly/reference/procedural-macros.html
##![allow(unused)]
#
#// extern crate proc; // 2018 版本之后不需要这行代码
#use proc::*;
#
#// 函数式
#make_answer!();
#
#// *** derive 式 ***
##[derive(AnswerFn)]
#struct A;
#
##[derive(HelperAttr)]
#struct B {
#    #[helper]
#    b: (),
#}
#// *** derive 式 ***
#
#// *** 属性式 ***
##[derive(HelperAttr)]
#struct C(#[helper] ());
#
#// Example: Basic function
##[show_streams]
#fn invoke1() {}
#// out: attr: ""
#// out: item: "fn invoke1() { }"
#
#// Example: Attribute with input
##[show_streams(bar)]
#fn invoke2() {}
#// out: attr: "bar"
#// out: item: "fn invoke2() {}"
#
#// Example: Multple tokens in the input
##[show_streams(multiple => tokens)]
#fn invoke3() {}
#// out: attr: "multiple => tokens"
#// out: item: "fn invoke3() {}"
#
#// Example:
##[show_streams { delimiters }]
#fn invoke4() {}
#// out: attr: "delimiters"
#// out: item: "fn invoke4() {}"
#
#// *** 属性式 ***
#
#fn main() {
#    println!("{}", answer());
#    println!("{}", answer_derive());
#}
```


