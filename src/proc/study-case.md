# 案例

## `assert_sync!` 

这是一个基础案例[^assert_sync1]，目的是在编译期间[^assert_sync2]测试某个类型是否实现了 `Sync`。

[`Sync`](https://doc.rust-lang.org/std/marker/trait.Sync.html) trait
表示该类型能在不同的线程之间安全地共享引用。当编译器确定某类型合适的话，那么会自动实现 `Sync`。

这里基于最基础的语法：声明泛型[结构体](https://doc.rust-lang.org/nightly/reference/items/structs.html)来实现编译期断言。

```rust,ignore
// 只需声明一个 Unit 结构体，附加 where 语句即可
// 技巧：在没有使用泛型参数的情况下也可以使用 where 语句
struct A where SomeType: SomeTrait;
```

利用声明宏和过程宏，完成目的：


```rust,ignore
// 声明宏
macro_rules! assert_sync_dcl {
    ($t:ty) => {{
        struct _AssertSync where $t: Sync;
    }};
}

// 过程宏
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned};

#[proc_macro]
pub fn assert_sync_proc(t: TokenStream) -> TokenStream {
    let ty = TokenStream2::from(t);
    TokenStream::from(quote! {{struct _AssertSync where #ty: Sync;}})
}

// 除了 Span 之外，与 `assert_sync_proc` 等价（但添加了一些调试打印）
#[proc_macro]
pub fn assert_sync_proc_spanned(t: TokenStream) -> TokenStream {
    use syn::spanned::Spanned;
    let ty = TokenStream2::from(t);
    let assert_sync = quote_spanned! {ty.span()=>
        {struct _AssertSync where #ty: Sync;}
    };
    // dbg!(&ty);
    // println!("{}", assert_sync);
    TokenStream::from(assert_sync)
}

// 使用断言
fn main() {
    assert_sync_proc!(Vec<u8>);
    // assert_sync_proc!(std::rc::Rc<u8>);

    assert_sync_proc_spanned!(Vec<u8>);
    // assert_sync_proc_spanned!(std::rc::Rc<u8>);

    assert_sync_dcl!(Vec<u8>);
    // assert_sync_dcl!(std::rc::Rc<u8>);
}
```

取消注释的那部分，会在编译时看到以下一条错误信息（过程宏错误信息可点击右上角取消隐藏看到）：

```rust,ignore
// 声明宏错误信息
error[E0277]: `Rc<u8>` cannot be shared between threads safely
  --> src/main.rs:5:9
   |
5  | /         struct _AssertSync
6  | |             where $t: Sync;
   | |___________________________^ `Rc<u8>` cannot be shared between threads safely
...
24 |       assert_sync_dcl!(std::rc::Rc<u8>);
   |       --------------------------------- in this macro invocation
   |
   = help: the trait `Sync` is not implemented for `Rc<u8>`
   = help: see issue #48214
   = help: add `#![feature(trivial_bounds)]` to the crate attributes to enable
   = note: this error originates in the macro `assert_sync_dcl` (in Nightly builds, run with -Z macro-backtrace for
more info)

#// 过程宏错误信息：不指定 Span
#error[E0277]: `Rc<u8>` cannot be shared between threads safely
#  --> src/main.rs:16:5
#   |
#16 |     assert_sync_proc!(std::rc::Rc<u8>);
#   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ `Rc<u8>` cannot be shared between threads safely
#   |
#   = help: the trait `Sync` is not implemented for `Rc<u8>`
#   = help: see issue #48214
#   = help: add `#![feature(trivial_bounds)]` to the crate attributes to enable
#   = note: this error originates in the macro `assert_sync_proc` (in Nightly builds, run with -Z macro-backtrace for
# more info)

#// 过程宏错误信息：指定 Span 可以更清楚地知道错误区域
#error[E0277]: `Rc<u8>` cannot be shared between threads safely
#  --> src/main.rs:20:31
#   |
#20 |     assert_sync_proc_spanned!(std::rc::Rc<u8>);
#   |                               ^^^^^^^^^^^^^^^ `Rc<u8>` cannot be shared between threads safely
#   |
#   = help: the trait `Sync` is not implemented for `Rc<u8>`
#   = help: see issue #48214
#   = help: add `#![feature(trivial_bounds)]` to the crate attributes to enable
```

声明宏的错误信息显然能够定位到声明宏所定义的地方，但过程宏的错误信息只定位到它被使用的地方，而且 Span
范围越小，就越清晰地指明错误的关键。

两种方法都能把主要的错误准确报告出来：`Rc<u8> cannot be shared between threads safely`。

[^assert_sync1]: 该例子受 [`quote::quote_spanned!`](https://docs.rs/quote/latest/quote/macro.quote_spanned.html#example) 
文档的启发。

[^assert_sync2]: 在编译时做出断言是这个例子的另一大亮点，利用类似的技巧，可以做出很多静态断言，参考 
[`static_assertions`](https://docs.rs/static_assertions/latest/static_assertions/) crate。
