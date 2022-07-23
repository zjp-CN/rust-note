# RFC 1414: static promotion

## 临时引用的静态生命周期提升

在 [RFC 1414: **rvalue static promotion**][RFC 1414] 中实现了右值的静态生命周期提升功能，也就是将
[常量表达式][constexpr] 的右值提升到静态内存 (static memory) 而不是放在栈槽 (stack slot) 中，并通过直接创建 
`'static` 引用来暴露这些值。典型例子是支持类似于下面的代码：

```rust
let x: &'static u32 = &42;
fn return_static_ref() -> &'static u32 { &42 }

// 可把 u32 换做任意数据结构
struct Custom(u32);
let x: &'static Custom = &Custom(42);
fn return_static_ref_custom() -> &'static Custom { &Custom(42) }

#use std::cell::Cell;
type Enum = Option<Cell<u32>>;
let x: &'static Enum = &None;
fn return_static__ref_none() -> &'static Enum { &None }
```

换言之，对于 `let x: &u32 = &...;` 数据 `...` 可能被放置在栈上[^&42]，而 `let x: &'static u32 = &...;` 则把数据 `...`
放置于静态内存，所以显然后者的生命周期可以比前者长，这对于返回 `&'static T` 的函数来说尤其有用。

[^&42]: 如果一个 `const fn` 返回 42，那么该常量函数返回的值不一定在静态区，见[最后部分](static-promotion.md#有资格的引用不一定被自动提升)。

此外，RFC 还总结了一条规则：

```rust
// 点击右上角按钮，可以正常运行这段代码

#use std::mem::size_of;
#type T = usize;
#const CONST_EXPR: &'static usize = &size_of::<usize>();

// 如果这行编译通过
const X: &'static T = &CONST_EXPR;

// 那么这行也应该编译通过
let x: &'static T = &CONST_EXPR;
```

[RFC 1414]: https://rust-lang.github.io/rfcs/1414-rvalue_static_promotion.html

[constexpr]: https://doc.rust-lang.org/reference/const_eval.html#constant-expressions

## 不是所有引用都有资格被提升

首先，最重要的前提是，此功能只针对 **常量表达式的值**：常见的常量表达式来源有 literals、constants、statics、`const fn`。

其次，在 RFC 中提到了一种特殊情况：

常量表达式中如果含 `UnsafeCell { ... }` 的构造表达式，那么临时的引用不能被提升为静态生命周期引用。

```rust
#use std::cell::{Cell, UnsafeCell};

// error[E0515]: cannot return reference to temporary value
fn not_allowed() -> &'static Option<UnsafeCell<u32>> { &Some(UnsafeCell::new(42)) }

// Cell::new(...) 背后使用了 UnsafeCell，所以也无法被提升
fn not_allowed_too() -> &'static Option<Cell<u32>> { &Some(Cell::new(42)) }
```

这是因为要确保被提升的值在引用后是真正不可变的。

## 在常量中出现 panic

另一个 [RFC 2345: const panic](https://rust-lang.github.io/rfcs/2345-const-panic.html) 规定了
`panic!` 可用于 constants、`const fn` 等场合（基于它的各种宏 `assert*` 也可用于同样的场合），并且
**其结果为编译时错误**。

所以，如果碰到 `panic!` 的常量表达式，临时的引用无法提升为 `'static`：

```rust
const fn size_of<T>() -> usize { panic!() }

// RFC 2345 规定的编译失败
// error[E0080]: evaluation of constant value failed
const X: &'static usize = &size_of::<()>();

// 编译失败，无法提升为 'static
// error[E0716]: temporary value dropped while borrowed
let x: &'static usize = &size_of::<()>();
```

注意，static promotion 所涉及的是引用（准确说是共享引用 `&T`）提升生命周期的问题：

```rust
const fn size_of<T>() -> usize { panic!() }

// RFC 2345 规定的编译失败
// error[E0080]: evaluation of constant value failed
const X: usize = size_of::<()>();

// 下面的代码与 static promotion 无关
// 编译通过，但运行失败
let x: usize = size_of::<()>();
let x: &usize = &size_of::<()>();
```

## 有资格的引用不一定被自动提升

这部分内容似乎没有在 RFC 中被提及。

从一个简单的例子出发。

```rust
use std::mem::size_of;

// 下面两行代码符合第一部分提到的规则
const _: &'static usize = &size_of::<usize>();
let _: &'static usize = &size_of::<usize>();
```

但是，考虑以下内容（下面对 static promotion 简称为“静态提升”）：

```rust,editable
// 此代码块可直接编辑

fn main() {
    // 代码正常
    const _: &'static usize = &size_of::<usize>();

    // 下面的代码却 **不符合** 第一部分提到的规则
    // error[E0716]: temporary value dropped while borrowed
    let _: &'static usize = &size_of::<usize>();
}

// 对 std::mem::size_of 进行封装，却无法“静态提升”
const fn size_of<T>() -> usize { std::mem::size_of::<T>() }
// 甚至对于简单的 literal，也无法“静态提升”
// const fn size_of<T>() -> usize { 0 }
```

回顾第一部分 RFC 提到的规则：

```rust,ignore
// 如果这行编译通过
const X: &'static T = &CONST_EXPR;

// 那么这行也应该编译通过
let x: &'static T = &CONST_EXPR;
```

`const _: &'static usize = &size_of::<usize>();` 这行说明 `&size_of::<usize>()` 是有资格进行“静态提升”的，而编译器对
`let _: &'static usize = &size_of::<usize>();` 没能把常量函数 `size_of`
的返回值视为常量，而是视为一个普通函数的值，所以这个值被放在了栈上，从而是“临时的”，而不是“静态的”。

事实上，对于 `const fn`，除非它被赋予 `#[rustc_promotable]` 属性，否则无法被隐式提升（等会解释这个概念）：

`std::mem::size_of` 具有这个属性，而任何自定义（包括封装）的 `size_of` 不具有。

有一些方法可以解决，基本逻辑是 **把常量函数的返回值显式地告知编译器为常量**。

注意：
* `let _: &'static T` 与 `fn f() -> &'static T` 在静态提升中差不多，都是把临时引用提升为静态引用。
* 链接的例子基于一个关于 [vtable 构造技巧的帖子](https://users.rust-lang.org/t/custom-vtables-with-integers)，
  本文也是受这则帖子及其讨论梳理出来的。

### 方法一：`const` items

```rust,editable
// 此代码块可直接编辑

fn main() {
    const X: usize = size_of::<usize>();
    // 当然也可以：
    // const X: &'static usize = &size_of::<usize>();
    let _: &'static usize = &X;
}

const fn size_of<T>() -> usize { std::mem::size_of::<T>() }
// const fn size_of<T>() -> usize { 0 }
```

这其实是实施静态提升功能前的做法，所以这是最基本的办法。

缺点在于，如果碰到 `fn f<T>() -> &'static ...` 这类带类型参数的函数，其函数内部的 constants 无法直接使用类型参数（
[vtable 例子](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=0abfbff9738af2fcfb4f36b310c51fe8) ）。

```rust,editable
// 此代码块可直接编辑

fn f<T>() -> &'static usize {
    // error[E0401]: can't use generic parameters from outer function
    const X: usize = size_of::<T>();
    &X
}

const fn size_of<T>() -> usize { std::mem::size_of::<T>() }
// const fn size_of<T>() -> usize { 0 }
fn main() { }
```

### 方法二：`associated constants`

关联常量有两类定义：`impl Type` 中、`impl Trait for Type` 中。

```rust,editable
// 此代码块可直接编辑

fn f<T>() -> &'static usize {
    struct Type<T>(*mut Self);
    impl<T> Type<T> {
        const X: usize = size_of::<T>();
    }
    &Type::<T>::X
}

const fn size_of<T>() -> usize { std::mem::size_of::<T>() }
// const fn size_of<T>() -> usize { 0 }
fn main() { }
```

```rust,editable
// 此代码块可直接编辑

fn f<T>() -> &'static usize {
    trait Trait<T> {
        const X: usize = size_of::<T>();
    }
    struct Type;
    impl<T> Trait<T> for Type {}
    &<Type as Trait<T>>::X
}

const fn size_of<T>() -> usize { std::mem::size_of::<T>() }
// const fn size_of<T>() -> usize { 0 }
fn main() {}
```

这解决了方法一中的常量无法使用类型参数的问题。（vtable 例子 [type-assoc-const] 和 [trait-assoc-const]）

[type-assoc-const]: https://users.rust-lang.org/t/custom-vtables-with-integers/78508/5

[trait-assoc-const]: https://users.rust-lang.org/t/custom-vtables-with-integers/78508/2

### 方法三： `const` block

```rust,editable
// 此代码块可直接编辑，且需要 nightly rustc

#![feature(inline_const)]

fn f<T>() -> &'static usize { const { &size_of::<T>() } }

const fn size_of<T>() -> usize { std::mem::size_of::<T>() }
// const fn size_of<T>() -> usize { 0 }
fn main() {}
```

这是最简洁的方法。详见 [RFC 2920]。（vtable [例子](https://users.rust-lang.org/t/custom-vtables-with-integers/78508/7)）

[RFC 2920]: https://rust-lang.github.io/rfcs/2920-inline-const.html

# RFC 3027: infallible promotion

[RFC 3027]: https://rust-lang.github.io/rfcs/3027-infallible-promotion.html

在 [RFC 3027] 中，规定了以下术语：
* 生命周期延长 (lifetime extension)：使引用具有 `'static` 生命周期
* 常量提升 (promotion)：将部分代码提取为常量的底层机制（这个过程是由编译器自动完成的，无需手写）

它们之间的关系是：生命周期延长（即 RFC 1414）是提升的一个主要应用，如果从用户的语法层面看，可以视为：

```rust
let _: &'static u32 = &42; // lifetime extension
let _: &'static u32 = {
    const X: u32 = 42; // promotion
    &X // lifetime extension
};
```

这是一个相当复杂的 RFC（涉及许多细节问题），并且还在进展中。

与之密切相关的 [const-eval] 文档进一步阐述了提升的限制，这被称作 [promotability]，比如
1. 必须能把表达式放入所谓的 [const]：所以不允许比较指针、不允许内部可变性，还要注意 `Send`、`Drop`、`static mut` 等问题
2. panic、overflow、边界检查不通过等情况下，不能被提升
3. 要满足 [const safety]
4. 以及其他一些规则

[const-eval]: https://github.com/rust-lang/const-eval

[promotability]: https://github.com/rust-lang/const-eval/blob/master/promotion.md#promotability

[const]: https://github.com/rust-lang/const-eval/blob/master/const.md

[const safety]: https://github.com/rust-lang/const-eval/blob/master/const_safety.md

回到 RFC 3027，其中区分了两种提升：

- 显式提升 (explicit promotion)：指必须提升成常量，比如 `#[rustc_args_required_const]` 和 `asm!`
- 隐式提升 (implicit promotion)：指可以提升成常量，比如引用不必是 `'static`、数组的 repeat expression 是 `Copy` 或者 repeat count 不超过 1 时

显式提升必须在编译时知道该值，因此如果无法确定该值，停止编译是正确的行为。

而隐式提升的典型案例是 `const fn` 必须在具有 `#[rustc_promotable]` 属性的情况下才具备隐式提升的资格。（见 [有资格的引用不一定被自动提升] 一节）

此外，隐式提升的一个前提是，表达式必须不能求值失败。这方面有诸多限制，首先需要区分哪些操作一定不会失败，哪些操作可能导致失败。（这在 RFC 中具体罗列了，这里不重复列了）

最终的部分结果是：
1. （自定义） `const fn` 因不具有 `#[rustc_promotable]` 而不具备隐式提升的资格，必须放入常量中明确要求编译期求值
2. 除法[^Division]、取模、索引操作不具备隐式提升的资格，也必须放入常量中明确要求编译期求值

解决方案仍然可以参照 [有资格的引用不一定被自动提升] 一节。

[有资格的引用不一定被自动提升]: static-promotion.md#有资格的引用不一定被自动提升

[^Division]: 小细节补充：对于简单的 `x/y`，只要 `y` 不为 0 的整数字面值，其结果依然可以被提升，见 [pull/80579](https://github.com/rust-lang/rust/pull/80579)：
```rust
let _: &'static i32 = &(1 / 2); // ok: 隐式提升
let _: &'static i32 = &((1-1) / 2); // ok: 隐式提升

const _: &'static i32 = &(1 / (1 + 1)); // ok: 明确要求编译期求值

// error[E0716]: temporary value dropped while borrowed
let _: &'static i32 = &(1 / (1 + 1)); // y 需要进一步计算，不支持隐式提升
```
