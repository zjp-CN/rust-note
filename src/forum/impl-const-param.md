# 针对常量泛型参数的分类实现

## 背景与问题

`const` 在 Rust 中是一个关键字，而且总是围绕着常量表达式 ([constant expressions]) 和编译期求值等话题。

而论及泛型参数 ([generic parameters])，我们总是想到 trait bounds
和生命周期。或者有时候，我们完全没注意到“泛型参数”这个描述。

我们知道，函数参数是列在函数名之后的 `(...)` 内的部分，而泛型参数是列在 `<...>` 内的部分。

泛型参数分为三类：
1. 生命周期参数
2. 类型参数
3. [常量参数][const generics parameters]

而且它们的顺序被规定为：生命周期必须放置于后两类之前，后两类可以交叉摆放。

对于用途最广泛的类型参数，常常利用 trait bounds 来限制实现，比如以下代码虽然声明一个泛型 `T`，
但只对 `T: Clone` 的情况实现功能。 

```rust
struct Item<T>(T);

impl<T: Clone> Item<T> {
    fn clone_myself(&self) -> Self {
        Item(self.0.clone())
    }
}
```

而常量参数通常是具体类型，目前仅允许一些基本类型
`u8`、`u16`、`u32`、`u64`、`u128`、`usize`、`i8`、`i16`、`i32`、`i64`、`i128`、`isize`、`char`、`bool`
作为常量参数。

比如对于 `struct Item<const I: i32>`，如果我们需要对 `I == 0` 和 `I != 0` 两种情况做不同的实现，该怎么做呢？

```rust
struct Item<const I: i32>;

// 当然不是以下做法，因为 Rust 不支持
impl<const I: i32> Item<I> where I == 0 {}
impl<const I: i32> Item<I> where I != 0 {}
```

## 常量泛型参数

常量泛型参数 ([const generics parameters])：
1. 可以在任何 [常量条目][const items] 中使用，而且只能独立使用，通常作为某类型的参数出现。
2. 作为一种常量上下文 (const context)，只与常量表达式和常量函数共存，无法与普通表达式一起使用。
3. 除非是单路径（单个标识符）或 literal，它必须使用 `{ ... }` 块表达式的形式。
4. 在单态化之后计算值，这与关联常量 ([associated constants]) 类似。

“单态化”在常量泛型参数中是一个基本视角，这意味着对于 `Item<const I: i32>`，单态化之后的
`Item<const I = 0>` 和 `Item<const I = 1>` 被认为是两个完全不同的类型。

而且 trait bounds 并不会考虑常量泛型参数的穷尽，Reference 给了以下一个例子：

```rust
struct Foo<const B: bool>;
trait Bar {}
impl Bar for Foo<true> {}
impl Bar for Foo<false> {}

fn needs_bar(_: impl Bar) {}
fn generic<const B: bool>() {
    let v = Foo::<B>;
    needs_bar(v); // ERROR: trait bound `Foo<B>: Bar` is not satisfied
}
```

所以，直接应用 trait bounds 似乎是一个不好的主意。

## `I` 和 `I == 0`

从泛型角度看， `struct Item<const I: i32>;` 定义了一个具体类型的泛型参数，但并不限定这个值。

所以，如果希望对所有值实现相同的功能，直接写下面的代码就行：

```rust
struct Item<const I: i32>;
impl<const I: i32> Item<I> {
    fn fun_for_all_i32() {}
    fn for_all_i32(self) {}
}

Item::<0>::fun_for_all_i32();
Item::<1>::fun_for_all_i32();
```

此外，单态化意味着我们可以对具体一种值实现单独的功能，所以 `I == 0` 的情况迎刃而解了：

```rust
#struct Item<const I: i32>;
impl Item<0> {
    fn fun_for_0() {}
    fn for_0(self) {}
}

Item::<0>::fun_for_0();
Item::<1>::fun_for_0(); // Error
```

Rust 为不存在的实现提供了良好的错误报告：

```rust,ignore
error[E0599]: no function or associated item named `fun_for_0` found for struct `Item<1_i32>` in the current scope
  --> src/main.rs:11:12
   |
4  | struct Item<const I: i32>;
   | -------------------------- function or associated item `fun_for_0` not found for this
...
11 | Item::<1>::fun_for_0(); // Error
   |            ^^^^^^^^^ function or associated item not found in `Item<1_i32>`
   |
   = note: the function or associated item was found for
           - `Item<0_i32>`
```

## `I != 0`

[@Michael Bryan](https://users.rust-lang.org/t/const-generics-how-to-impl-not-equal/74946/4)
提供了一种思路：泛型常量表达式 + trait bounds。

[`#![feature(generic_const_exprs)]`](https://github.com/rust-lang/rust/issues/76560)
允许你写出良好形式 ([well-formedness]) 的常量泛型表达式，并且进行常量求值，没有这个功能，
Rust 只允许 `I` 或者 `{ I }` 这种“简单形式”的表达式。

`I != 0` 是一种良好的形式（当然，常量函数调用也是一种良好的形式），所以我们可以这样写：

```rust,editable
// 点击右上角就可以运行代码；你可以直接在网页中编辑这段代码
#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

struct Item<const I: i32> {}

impl<const I: i32> Item<I>
where
    [(); (I != 0) as usize - 1]:, // 这里的技巧在常量表达式中非常常见
{
    fn for_non_zero() {}
}

fn main() {
    Item::<1>::for_non_zero();
    // 下面这一行代码导致编译错误
    Item::<0>::for_non_zero();
}
```

从功能上看，它的确解决了问题 —— 即使我们误入不存在的函数/方法，编译器会帮助我们的：

```rust,ignore
error[E0284]: type annotations needed: cannot satisfy `the constant `Item::<{_: i32}>::{constant#0}` can be evaluated`
  --> src/main.rs:17:5
   |
17 |     Item::for_non_zero();
   |     ^^^^^^^^^^^^^^^^^^ cannot satisfy `the constant `Item::<{_: i32}>::{constant#0}` can be evaluated`
   |
note: required by a bound in `Item::<I>::for_non_zero`
  --> src/main.rs:9:10
   |
9  |     [(); (I != 0) as usize - 1]:, // 这里的技巧在常量表达式中非常常见
   |          ^^^^^^^^^^^^^^^^^^^^^ required by this bound in `Item::<I>::for_non_zero`
10 | {
11 |     fn for_non_zero() {}
   |        ------------ required by a bound in this
```

上面报告的错误显然不直观，我们可以施加技巧，利用类型和 trait 让错误更直观一些（虽然很间接得到）：

```rust,editable
#![feature(generic_const_exprs)]
#![allow(incomplete_features)]
#![feature(negative_impls)]

struct Item<const I: i32>;

impl<const I: i32> Item<I>
where
    Check<{ I != 0 }>: NonZero,
{
    fn for_non_zero() {}
}

struct Check<const C: bool>;
trait NonZero {}
impl NonZero for Check<true> {}
impl !NonZero for Check<false> {} // 这一步在这里并不是必要的

fn main() {
    Item::<1>::for_non_zero();
    // Error:
    Item::<0>::for_non_zero();
}
```

```rust,ignore
error[E0599]: the function or associated item `for_non_zero` exists for struct `Item<0_i32>`, but its trait bounds were not satisfied
  --> src/main.rs:22:16
   |
5  | struct Item<const I: i32>;
   | -------------------------- function or associated item `for_non_zero` not found for this
...
14 | struct Check<const C: bool>;
   | ---------------------------- doesn't satisfy `Check<{ I != 0 }>: NonZero`
...
22 |     Item::<0>::for_non_zero();
   |                ^^^^^^^^^^^^ function or associated item cannot be called on `Item<0_i32>` due to unsatisfied trait bounds
   |
   = note: the following trait bounds were not satisfied:
           `Check<{ I != 0 }>: NonZero`
note: the following trait must be implemented
  --> src/main.rs:15:1
   |
15 | trait NonZero {}
   | ^^^^^^^^^^^^^^^^
```

## `I == 0` | `I > 0` | `I < 0`

如果我们想对上面的 `const I: i32` 做更多分类呢？或者在这些分类中，我们想要同样的函数名返回不同的类型呢？

我没有完美的答案，因为具体的需求会导致不同的代码设计。

我给出自己的思考结果：

1. 常量泛型参数无法拓展到自定义类型，所以需要围绕基本类型来实现；
2. 常量表达式总是意味着它的值必须在编译时知晓，所以它的来源很狭窄，唯有泛型函数帮助我们做更多事情。

```rust
struct Item<const U: u8>;
struct A; struct B; struct C;

impl Item<0> { fn foo() -> A { A } }
impl Item<1> { fn foo() -> B { B } }
impl Item<2> { fn foo() -> C { C } }

const fn check(i: i32) -> u8 {
    match i {
        0   => 0,
        1.. => 1,
        _   => 2,
    }
}

Item::<{ check(0) }>::foo();  // A
Item::<{ check(1) }>::foo();  // B
Item::<{ check(-1) }>::foo(); // C
```

如果你使用上一小节提到的技巧，目前是无法实现同名函数的：

```rust
// error[E0592]: duplicate definitions with name `f`
// error[E0080]: evaluation of `main::Foo::{constant#0}` failed
#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

struct Foo<const I: i32 = 0> {}

impl<const I: i32> Foo<I> where [(); (I < 0) as usize - 1]:,
{ fn f() {} }

impl<const I: i32> Foo<I> where [(); (I > 0) as usize - 1]:,
{ fn f() {} }

impl Foo<0>
{ fn f() {} }
```

```rust
// error[E0592]: duplicate definitions with name `f`
#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

struct Foo<const I: i32> {}

impl<const I: i32> Foo<I> where Check<{ check(I) }>: Greter,
{ fn f() {} }

impl<const I: i32> Foo<I> where Check<{ check(I) }>: Less,
{ fn f() {} }

impl<const I: i32> Foo<I> where Check<{ check(I) }>: Equal,
{ fn f() {} }
#
#const fn check(i: i32) -> u8 {
#    match i {
#        0 => 0,
#        1.. => 1,
#        _ => 2,
#    }
#}
#
#struct Check<const C: u8>;
#trait Equal {}
#trait Greter {}
#trait Less {}
#impl Equal for Check<0> {}
#impl Greter for Check<1> {}
#impl Less for Check<2> {}
```

## 参考资料

1. Rust User Forum: [Const generics: how to impl “not equal”](https://users.rust-lang.org/t/const-generics-how-to-impl-not-equal/74946/8)


[constant expressions]: https://doc.rust-lang.org/reference/const_eval.html
[generic parameters]: https://doc.rust-lang.org/reference/items/generics.html#const-generics
[const generics parameters]: https://doc.rust-lang.org/reference/items/generics.html#const-generics
[const items]: https://doc.rust-lang.org/reference/items/constant-items.html
[associated constants]: https://doc.rust-lang.org/reference/items/associated-items.html?highlight=monom#associated-constants
[well-formedness]: https://hackmd.io/OZG_XiLFRs2Xmw5s39jRzA?view
