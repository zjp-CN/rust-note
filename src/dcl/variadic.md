# “变长参数”函数与回调

## 前言

Rust 中，“变长参数” (variadic) 总是离不开宏这个话题：众所周知，Rust 的宏有[三个主要功能][RBE: macros]
1. 减少样板代码
2. 自定义语法 （DSL）
3. 变长参数接口

[RBE: macros]: https://doc.rust-lang.org/stable/rust-by-example/macros.html

作为静态语言的 Rust，函数参数的个数在声明时已经被固定，也不可能传入不同个数的参数。

但我们真的无法针对函数设计出“变长参数”吗？

## 背景

首先，明确我们需要什么[^background]。下面是一个略为复杂的 API 设计：

```rust,ignore
memoize(&mut ui, comp_, (...), |_| {});
// memoize : 提供给使用者的函数
// &mut ui : 函数的固定参数，或者设计者认为非常重要的参数（在这篇文章中不重要）
// comp_   : 使用者或者编写者提供的函数（或方法），是本文讨论的回调函数
// (...)   : 回调函数的参数，对 memoize 来说是“变长的”（是本文的重点）
// |_| {}  : 使用者提供的闭包，它可以提供上下文变量（在这篇文章中不重要）
```

使用者可以这样调用：

```rust,ignore
memoize(&mut ui, comp2, (2, 3), |_| {});
memoize(&mut ui, comp3, (1, 2, 3), |_| {});
memoize(&mut ui, comp4, (0, 1, 2, 3), |_| {});
```

这里，我们用元组这个看似可变长度的数据结构（而且它可以同时容纳不同的类型），来表达回调函数所需的（部分）参数。

而变长参数被设计成需满足 `PartialEq + Clone + 'static`，即：

```rust,ignore
pub fn memoize<Params, Content, Comp>(ui: &mut Ui, component: Comp, params: Params, content: Content)
    where Params: PartialEq + Clone + 'static,
          Content: FnOnce(&mut Ui),
          Comp: todo!() { todo!() }

fn comp2(ui: &mut Ui, a: u8, b: u32, f: impl FnOnce(&mut Ui)) { f(ui); }
fn comp3(ui: &mut Ui, a: u8, b: u32, c: u64, f: impl FnOnce(&mut Ui)) { f(ui); }
fn comp4(ui: &mut Ui, a: u8, b: u32, c: u64, d: usize, f: impl FnOnce(&mut Ui)) { f(ui); }
```

忽略 `&mut Ui` 和 `f: impl FnOnce(&mut Ui)` 这两个“固定参数”，它们不是本文的重点。

[^background]: 原例子由 @费超 [提供](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=4ed74b052309f1bd51bc4e9a1d8872cb)。

## 思路

在 Rust 的类型系统中，使用 trait 进行抽象是最根本的方式：

```rust,ignore
pub trait Component<Params, Content> {
    fn call(&self, ui: &mut Ui, params: Params, content: Content);
}
```

这里的关键在于，使用`Params` 泛型参数来抽象 `comp*` 函数中这些 `a, b, c, ...` 函数参数。

然后，给函数实现 `Component` （以简单的两参数为例）：

```rust,ignore
impl<F, P1, P2, Content> Component<(P1, P2), Content> for F
where
    P1: PartialEq + Clone + 'static,
    P2: PartialEq + Clone + 'static,
    Content: FnOnce(&mut Ui),
    F: Fn(&mut Ui, P1, P2, Content),
{
    fn call(&self, ui: &mut Ui, params: (P1, P2), content: Content) {
        let (p1, p2) = params;
        self(ui, p1, p2, content)
    }
}
```

这个 `where` 语句表达了最核心的内容：
1. 原本的泛型参数 `Params` 被“规定成”某种严格/具体一些的形式：`(P1, P2)`。`(P1, P2)` 依然是一种泛型，表示使用者应传入两元素的元组。
2. `F: Fn(&mut Ui, P1, P2, Content)`：这把所考虑的回调函数的形式抽象出来了。
3. `P1`、`P2`、`Content` 的 trait bounds 不是本文的重点，无需赘言。

```rust
// https://play.rust-lang.org/?version=nightly&mode=debug&edition=2021&gist=820a56b25ce4cfef20795e0b111f7cc5
#![allow(unused)]

#pub struct Ui {}
#impl Ui {
#    pub fn update<T>(&mut self, a: String, b: Vec<T>, f: impl FnOnce(&mut Ui))
#        where T: PartialEq + Clone + 'static {
#    }
#}
#
#pub trait Component<Params, Content> {
#    fn call(&self, ui: &mut Ui, params: Params, content: Content);
#}
#
#impl<F, P1, P2, Content> Component<(P1, P2), Content> for F
#    where P1: PartialEq + Clone + 'static,
#          P2: PartialEq + Clone + 'static,
#          Content: FnOnce(&mut Ui),
#          F: Fn(&mut Ui, P1, P2, Content)
#{
#    fn call(&self, ui: &mut Ui, params: (P1, P2), content: Content) {
#        let (p1, p2) = params;
#        self(ui, p1, p2, content)
#    }
#}
#
pub fn memoize<Params: PartialEq + Clone + 'static,
               Content: FnOnce(&mut Ui),
               Comp: Component<Params, Content>>(
    ui: &mut Ui, component: Comp, params: Params, content: Content) {
    component.call(ui, params, content);
}

fn comp2(ui: &mut Ui, a: u8, b: u32, f: impl FnOnce(&mut Ui)) { f(ui); }

fn main() {
    let mut ui = Ui {};

    memoize(&mut ui, comp2, (2, 3), |_| {});

    let args = (String::new(), vec![(1usize, 1.0f64)]);
    memoize(&mut ui, Ui::update, args, |_| {});
}
```

至此，`memoize` 支持任何具有 `F: Fn(&mut Ui, P1, P2, Content)` 形式的函数（方法、甚至闭包），只要其参数满足各自的 trait bounds。

如何支持其他参数长度的函数 `F: Fn(&mut Ui, ..., Content)` 呢？

很简单，却也很无聊，只需继续实现 `impl<F, ..., Content> Component<(...), Content> for F` —— 整个过程只运用了最基础的 trait bounds 知识。

## 进阶

根据上一部分的内容，我们得到一份“样板代码”，你可以手写需要的部分，但支持宏编程的语言一定不会按这种原始的方式拓展样板代码。考虑以下宏：

```rust
// https://play.rust-lang.org/?version=nightly&mode=debug&edition=2021&gist=b35db5a10a259e9ac1cdd1899f781ab1
#![allow(unused)]
#
#pub struct Ui {}
#
#pub trait Component<Params, Content> {
#    fn call(&self, ui: &mut Ui, params: Params, content: Content);
#}

macro_rules! impl_component {
    ($($P:ident),*) => {
        impl<F, $($P,)* Content> $crate::Component<( $($P,)* ), Content> for F
            where F: Fn(&mut Ui, $($P,)* Content),
                  $( $P: ::std::cmp::PartialEq + ::std::clone::Clone + 'static, )*
                  Content: ::std::ops::FnOnce(&mut Ui)
        {

            fn call(&self, ui: &mut Ui, params: ( $($P,)* ), content: Content) {
                #[allow(non_snake_case)]
                let ($($P,)*) = params;
                self(ui, $($P,)* content)
            }
        }
    };
}

impl_component!();
impl_component!(P1);
impl_component!(P1, P2);
impl_component!(P1, P2, P3);
impl_component!(P1, P2, P3, P4);
#
#pub fn memoize<Params, Content, Comp>(ui: &mut Ui, component: Comp, params: Params, content: Content)
#    where Params: PartialEq + Clone + 'static,
#          Content: FnOnce(&mut Ui),
#          Comp: Component<Params, Content>
#{
#    component.call(ui, params, content);
#}
#
#fn comp1(ui: &mut Ui, a: u8, f: impl FnOnce(&mut Ui)) { f(ui); }
#fn comp_(ui: &mut Ui, a: &str, f: impl FnOnce(&mut Ui)) { f(ui); }
#fn comp2(ui: &mut Ui, a: u8, b: u32, f: impl FnOnce(&mut Ui)) { f(ui); }
#fn comp3(ui: &mut Ui, a: u8, b: u32, c: u64, f: impl FnOnce(&mut Ui)) { f(ui); }
#fn comp4(ui: &mut Ui, a: u8, b: u32, c: u64, d: usize, f: impl FnOnce(&mut Ui)) { f(ui); }

fn main() {
    let mut ui = Ui {};
    memoize(&mut ui, comp1, (1,), |_| {});
    memoize(&mut ui, comp_, ("",), |_| {});
    memoize(&mut ui, comp2, (2, 3), |_| {});
    memoize(&mut ui, comp3, (1, 2, 3), |_| {});
    memoize(&mut ui, comp4, (0, 1, 2, 3), |_| {});
}
```

这个声明宏十分简单，因为只匹配了一连串逗号分隔的标识符，然后利用反复技巧去展开。

如果你想精简重复的宏调用，或许希望继续写一个宏 `all_tuples!`，然后通过 `all_tuples! {impl_component, 1, 16, P}` 生成支持 1 ~ 16 个函数参数的实现。

这实际上是 [`bevy::SystemParamFunction`] 的基本思路。如果你已经理解以上内容，那么可以轻松阅读和理解 [那部分源码][SystemParamFunction-src]。

不过，[`bevy::all_tuples!`] 使用了过程宏来定义。你完全可以自己写一个类似过程宏，只需要运用简单的步骤，比如这样：

```rust,ignore
#use proc_macro::TokenStream;
#use quote::{format_ident, quote};
#use syn::{
#    parse::{Parse, ParseStream},
#    parse_macro_input, Ident, LitInt, Result,
#};
#
#[proc_macro]
pub fn all_tuples(input: TokenStream) -> TokenStream {
    let Input { name, start, end, ident } = parse_macro_input!(input);
    let id = |s: u8, e: u8| (s..e).map(|n| format_ident!("{ident}{n}"));
    let items = (start..=end).map(|n| {
                                 let ids = id(start, n + 1);
                                 quote!( #name!{#(#ids),*} )
                             });
    quote!(#(#items)*).into()
}

struct Input {
    name:  Ident,
    start: u8,
    end:   u8,
    ident: Ident,
}

impl Parse for Input {
    fn parse(input: ParseStream) -> Result<Self> {
        use syn::token::Comma;
        let name     = input.parse()?;
        let _: Comma = input.parse()?;
        let start    = input.parse::<LitInt>()?.base10_parse()?;
        let _: Comma = input.parse()?;
        let end:     = input.parse::<LitInt>()?.base10_parse()?;
        let _: Comma = input.parse()?;
        let ident    = input.parse()?;
        Ok(Input { name, start, end, ident })
    }
}
```

宏充满了技巧，这需要观察和练习。如果你感兴趣的话，使用这段代码的展开结果[见此处][expanded]。

然而，我想提醒你的是，本文的核心技巧是 trait 和泛型参数，宏只是锦上添花的内容。

[expanded]: https://play.rust-lang.org/?version=nightly&mode=debug&edition=2021&gist=1cee68df91a0d8654dfbe1c88ec5edf1
[`bevy::SystemParamFunction`]: https://docs.rs/bevy/latest/bevy/ecs/system/trait.SystemParamFunction.html
[SystemParamFunction-src]: https://github.com/bevyengine/bevy/blob/fed93a0edce9d66586dc70c1207a2092694b9a7d/crates/bevy_ecs/src/system/function_system.rs#L492-L541
[`bevy::all_tuples!`]: https://github.com/bevyengine/bevy/blob/fed93a0edce9d66586dc70c1207a2092694b9a7d/crates/bevy_ecs/macros/src/lib.rs#L48-L81

## 细节

注意：这里的细节与最佳实践无关，意图在于简述事实。

### 元组

函数 `memoize` 的 `params: Params` 参数实际是利用元组的以下特点做到“变长”的：
* 元组可以容纳不同类型的元素，这与函数的各个参数可以为不同类型一致，所以使用泛型 `(P1, ...)` 来抽象“函数参数各自具有其类型”；
* 这种“变长”的前提是“定长”：只有实现了固定长度的泛型元组，才能运用到相应的函数上。  
  即因为 `impl<F, P1, ..., Pn, Content> Component<(P1, ..., Pn), Content> for F`，所以
  `memoize(&mut ui, comp_n, (p1, ..., pn), |_| {});`。

这种抽象并不是完美的，它建立在类型系统之上：泛型 `(P1, ...)` 虽然解决了参数的形式问题，但也意味着可能的、复杂的 trait bounds。

还是以两参数为例，这里的实现相当精简：

```rust,ignore
impl<F, P1, P2, Content> Component<(P1, P2), Content> for F
where
    P1: PartialEq + Clone + 'static,
    P2: PartialEq + Clone + 'static,
    Content: FnOnce(&mut Ui),
    F: Fn(&mut Ui, P1, P2, Content),
```

但你容易忽略一些基本事实和局限性：
* 所有元素满足 trait bound，并不意味着元组满足这个 trait bound：如果要求 `P1: SomeTrait` ... `Pn: SomeTrait`，则必须 **手动** 给
  `(P1, ..., Pn)` 实现 `SomeTrait`。幸运的是，针对这种“传递”关系，标准库已经给元组，在最多 12 个元素的情况下实现了[某些][tuple-impls] traits。
  ```rust,ignore
  pub fn memoize<Params, Content, Comp>(ui: &mut Ui, component: Comp, params: Params, content: Content)
    where Params: PartialEq + Clone + 'static, // (P1, ..., Pn): PartialEq + Clone + 'static
          Content: FnOnce(&mut Ui),
          Comp: Component<Params, Content> { ... }
  ```
  超出标准库的元组实现，意味着你需要自己实现。参考 bevy 的 [`SystemParam`]、[`SystemParamFetch`]、[`SystemParamState`]。  
  例如，超出 12 个元素，且涉及标准库定义的 traits；或者无论多少元素，但凡涉及第三方库的 traits：你会遇到孤儿原则，需要使用其他技巧来手动实现。
* 换言之，即便使用 `all_tuples!(impl_component, 1, 16)` 之类的技巧生成最多支持 16 个元素的元组，但[以下代码][tuple-13]无法编译通过：
  ```rust,ignore
  fn comp13(ui: &mut Ui, p1: u8, p2: u8, p3: u8, p4: u8, p5: u8, p6: u8, p7: u8, p8: u8, p9: u8,
            p10: u8, p11: u8, p12: u8, p13: u8, f: impl FnOnce(&mut Ui)) { f(ui); }

  memoize(&mut ui, comp13, (0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0), |_| {});

  error[E0277]: can't compare `(u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8)` with `(u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8)`
   --> src/main.rs
    |
    | memoize(&mut ui, comp13, (0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0), | _ | {});
    | ^^^^^^^ no implementation for `(u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8) == (u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8)`
  ```

[`SystemParam`]: https://docs.rs/bevy/latest/bevy/ecs/system/trait.SystemParam.html#foreign-impls
[`SystemParamFetch`]: https://docs.rs/bevy/latest/bevy/ecs/system/trait.SystemParamFetch.html#foreign-impls
[`SystemParamState`]: https://docs.rs/bevy/latest/bevy/ecs/system/trait.SystemParamState.html#foreign-impls
[tuple-impls]: https://doc.rust-lang.org/std/primitive.tuple.html#trait-implementations-1
[tuple-13]: https://play.rust-lang.org/?version=nightly&mode=debug&edition=2021&gist=b5780dd7175419ada76d4268c6cd0026

总之，Rust 是静态语言，变长参数、可选参数这种极具动态语言的特性的事物对 Rust 来说并不是必须，但好在其类型系统不算薄弱。

### `'static`

在参数的 trait bound 中，`P: PartialEq + Clone + 'static` 约束描述了函数参数必须是 **不含引用的类型** 或者
**生命周期为 `'static` 的类型**[^static]，而且该类型实现了 `PartialEq` 和 `Clone` trait。

这意味着，`memoize` 虽然可以接受下面这个函数，但[不接受非 `'static` 生命周期的参数][non-ref]：

```rust,ignore
fn comp_(ui: &mut Ui, a: &str, b: Vec<&str>, f: impl FnOnce(&mut Ui)) { f(ui); }
// 实际上，由于 `P: 'static`，a 必须是 &'static str， b 必须是 Vec<'static str>
memoize(&mut ui, comp_, ("", vec![""]), |_| {}); // ok

let s = String::from(""); // 生命周期不是 'static
memoize(&mut ui, comp_, (&s, vec![&s]), |_| {}); // error[E0597]: `s` does not live long enough
```

当你[去掉 `'static` 约束][non-static]，`memoize` 接受上面的代码。

[^static]: 如果你不明白 `T: 'static` 意味着什么，请仔细阅读 [Common Rust Lifetime Misconceptions]

[Common Rust Lifetime Misconceptions]: https://github.com/pretzelhammer/rust-blog/blob/master/posts/common-rust-lifetime-misconceptions.md#2-if-t-static-then-t-must-be-valid-for-the-entire-program
[non-ref]: https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=145a70626c55695bcdad49e43603e05c
[non-static]: https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=145a70626c55695bcdad49e43603e05c

### `Fn` or `fn`

你可能会考虑到给什么样的函数实现 `Component`：

```rust,ignore
impl<F: FnOnce(&mut Ui, ...), ...> Component<...> for F { } // 或者
impl<F: FnMut(&mut Ui, ...), ...> Component<...> for F { } // 或者
impl<F: Fn(&mut Ui, ...), ...> Component<...> for F { } // 或者
impl<...> Component<...> for fn(&mut Ui, ...) { } // 函数指针
```

Rust 可以从多种维度分类和抽象函数：
1. 使用类型区分是否能捕获环境中的变量：
    * 不能捕获、不捕获：[Function Item]、[Function Pointer]
    * 捕获：[Closure]
2. 使用 trait 抽象“以何种方式捕获/或者如何处理捕获的环境变量”（注意，这与函数参数的所有权、可变性无关）：
    * [`FnOnce`]：获取捕获变量的所有权
    * [`FnMut`]：修改捕获变量的值；且 `FnMut` 是 `FnOnce` 的子类型
    * [`Fn`]：获取捕获变量的引用；且 `Fn` 是 `FnMut` 的子类型

它们的具体说明参考各自的文档，这里只总结一下基本知识：
* [Function Item]：只适用于函数条目，可以赋给变量，但无法写出其类型；可以转化成函数指针，而且实现了 `FnOnce`、`FnMut`、`Fn` 以及其他 traits，零大小
* [Function Pointer]：函数指针，是 primitive type，通常从函数条目或非捕获闭包使用 `as` 转化过来，对最多 12 个参数的情况实现了 
  `FnOnce`、`FnMut`、`Fn` 以及其他 traits，具有 `usize` 大小
* [Closure]：闭包，按照一些规则实现 `FnOnce`、`FnMut`、`Fn` 及其他 traits 中的部分或全部，可以赋给变量，但无法写出其类型；由捕获变量的数量决定大小

[Function Item]: https://doc.rust-lang.org/reference/types/function-item.html
[Function Pointer]: https://doc.rust-lang.org/reference/types/function-pointer.html
[Closure]: https://doc.rust-lang.org/reference/types/closure.html
[`FnOnce`]: https://doc.rust-lang.org/std/ops/trait.FnOnce.html
[`FnMut`]: https://doc.rust-lang.org/std/ops/trait.FnMut.html
[`Fn`]: https://doc.rust-lang.org/std/ops/trait.Fn.html

### 类型参数 or 关联类型

或许你想使用关联类型定义 `Params`：

```rust,ignore
pub trait Component<Content> {
    type Params: PartialEq + Clone + 'static;
    fn call(&self, ui: &mut Ui, params: Self::Params, content: Content);
}

// 或者
pub trait Component {
    type Params: PartialEq + Clone + 'static;
    type Content: FnOnce(&mut Ui);
    fn call(&self, ui: &mut Ui, params: Self::Params, content: Content);
}
```

无论那种做法，在本文所需的场景下都不合适：因为它们实际上没有抽象出“不限定函数参数的类型”。

我总结了以下表格，也可以参考 [某则帖子][generics_AT] 对具体的 `Iterator` 进行类型参数还是关联类型抽象的讨论。

| 角度                                 | 类型参数  `T`、`U`               | 关联类型 `T`、`U` （不考虑 GAT）                                                                          |
|--------------------------------------|----------------------------------|-----------------------------------------------------------------------------------------------------------|
| 形式                                 | `trait Component<T, U, ...>`     | 处于 trait 内的 `type T;` 或者 `type U: ...;`                                                             |
| Reference 链接                       | [generics]                       | [associated-types]                                                                                        |
| 泛型 -> 具体类型                     | 由使用者决定                     | 由 trait impl 决定（即由实现者决定）                                                                      |
| 抽象性                               | 满足 trait bound 的任意类型      | 从声明的角度看，是满足 trait bound 的任意类型；<br>但从实现的角度看，无抽象，因为关联类型因具体实现而确定 |
| 使用语法                             | 该 trait 中：直接使用 `T`、`U`   | `Self::T`、`Self::U`、`<Implementator as Trait>::T`                                                       |
| implementator[^implementator] vs `T` | 一对多[^1vs-]                    | 一对一[^1vs1]                                                                                                    |
| 类型推断                             | 通常要指明类型                   | 容易直接推断，因为一经实现，类型是固定的                                                                  |
| 方法（函数）                         | 泛型方法（由 trait bounds 提供） | 具体类型的方法（函数）                                                                                    |

[^implementator]: 实现 trait 的类型：比如 `impl AsRef<[u8]> for str` 中，`str` 就叫做 implementator。

[^1vs-]: 比如 `impl AsRef<[u8]> for str`、`impl AsRef<str> for str`、`impl AsRef<OsStr> for str` 等等。

[^1vs1]: 比如 `impl<'a> Iterator for Chars<'a>` 只有 `type Item = char`。

[generics]: https://doc.rust-lang.org/reference/items/generics.html
[associated-types]: https://doc.rust-lang.org/reference/items/associated-items.html#associated-types
[generics_AT]: https://users.rust-lang.org/t/differece-between-type-item-and-item

## 其他方式

完全使用宏（无须定义 `Component` trait），比如这样：

```rust
// https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=f44c02fd60a763d30fef5d25d29e7d45
#![allow(unused)]

pub struct Ui {}

macro_rules! memoize {
    ($ui:expr, $f:path, ($($p:expr),*), $content:expr) => {
        $f($ui, $($p,)* $content);
    };
}

fn comp0(ui: &mut Ui, f: impl FnOnce(&mut Ui)) { f(ui); }
fn comp_(ui: &mut Ui, a: &str, f: impl FnOnce(&mut Ui)) { f(ui); }
fn comp4(ui: &mut Ui, a: u8, b: u32, c: u64, d: usize, f: impl FnOnce(&mut Ui)) { f(ui); }
fn comp12(ui: &mut Ui, p1: u8, p2: u8, p3: u8, p4: u8, p5: u8, p6: u8, p7: u8, p8: u8, p9: u8,
          p10: u8, p11: u8, p12: u8, f: impl FnOnce(&mut Ui)) { f(ui); }

fn main() {
    let mut ui = Ui {};
    memoize!(&mut ui, comp0, (), |_| {});
    memoize!(&mut ui, comp_, (""), |_| {});
    memoize!(&mut ui, comp4, (0, 1, 2, 3), |_| {});
    memoize!(&mut ui, comp12, (0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0), |_| {});
}
```

甚至可以[用宏实现命名和默认参数][Default Arguments]。

此外，还有 2013 年提出，但已经搁置的 [RFC: variadic generics]。

[Default Arguments]: https://internals.rust-lang.org/t/named-default-arguments-a-review-proposal-and-macro-implementation
[RFC: variadic generics]: https://github.com/rust-lang/rfcs/issues/376

