# `LazyLock` 未稳定的难点

## `LazyLock` 的背景

翻看古老的教程，初始化数据用的最多的是 [`lazy_static!`](https://docs.rs/lazy_static/*/lazy_static/index.html#example)。

但实际上，它正逐渐在被 [`once_cell`](https://docs.rs/once_cell) 库替代：
* `lazy_static` 库已经存在了 10 年，截止目前下载量超过 1.56 亿次，不久前官方宣布 [落幕](https://github.com/rust-lang-nursery/lazy-static.rs/issues/214)
* `once_cell` 库仅存在 5 年，下载量就已突破 1.4 亿次，其作者就是 Rust Analyzer 的作者

`once_cell` 是受到 `lazy_static` 等库的启发而编写的，它的文档已经做了丰富的描述来介绍自己。

但概括地说，我觉得它们之间的区别主要在于：
* `lazy_static` 仅通过一个宏来生成与变量同名的空类型，使用者根据 `Deref` （即 `&*lazy`）来初始化并获取数据。
* `once_cell` 则通过一系列类型来抽象初始化，并提供方法来控制如何初始化和获取数据。可以说 `once_cell`
  是 `lazy_static` 的后继者，并且比后者可以做的事情更多，比如 `once_cell` 提供
  * 单线程和跨线程以及非阻塞三种情况的初始化
    * 单线程和跨线程的区别是基于线程安全，即 Rust 中是否实现 [`Sync`] trait —— 是否安全地在线程间使用共享引用来共享数据
    * 非阻塞考虑的是，多个线程同时初始化时，保证率先的那个初始化成功；从 API 来看
        * 阻塞式：`once_cell::sync::Lazy::new(function)` ，必须提供一个初始化函数来生成实例
        * 非阻塞式：`once_cell::race::OnceBox::new()` 无需立马提供函数，而是通过 `.get_or_init(function)` 设置初始化函数，
          并且这个调用可以多次、跨线程调用（因为 `OnceBox` 实现了 `Sync`）
  * 将实现的步骤分开，从而有 `OnceCell` 和 `Lazy` 两个类型
    * `OnceCell` 保证只写入一次，有一系列关于设置和读取值的方法，它是 `lazy_static` 的间接替代
    * `Lazy` 在 `OnceCell` 的基础上保证第一次读取时才写入（惰性初始化），它是 `lazy_static` 的直接替代
      * 注意，`Lazy` 还提供了 [`DerefMut`](https://docs.rs/once_cell/1.18.0/once_cell/unsync/struct.Lazy.html#impl-DerefMut-for-Lazy%3CT,+F%3E)，
        这与 `lazy_static` 和标准库的 [`LazyLock`] 不同，意味着你可以通过 `&mut *lazy` [修改它的值][mut]，从而实际上并非“只写入一次”
  * 一个与 `lazy_static` 在使用上最直观的区别是，`once_cell` 的数据结构不仅可用于 static，还可以放置于任何自定义的数据结构
    * `lazy_static` 适合初始化一个全局的、线程安全的值，其初始化函数在编译时已知，很少将它放入自定义的数据结构
    * `once_cell` 提供的类型可以在任意地方构建初始化函数，即初始化函数可以在编译时甚至运行时已知，完全可以放入自定义数据结构

一个好消息是，标准库正在参照 `once_cell` 库，将大部分功能实现。[`OnceCell`] 及其线程安全的 [`OnceLock`] 已在今年 6 月的 Rust [1.70] 的标准库中稳定。

但其惰性版本 [`LazyCell`]、[`LazyLock`] 尚未稳定，原因有几点：

* <https://github.com/rust-lang/rust/issues/109736>
* <https://github.com/matklad/once_cell/issues/167>

[`Sync`]: https://doc.rust-lang.org/std/marker/trait.Sync.html
[`OnceCell`]: https://doc.rust-lang.org/std/cell/struct.OnceCell.html
[`OnceLock`]: https://doc.rust-lang.org/std/sync/struct.OnceLock.html
[`LazyCell`]: https://doc.rust-lang.org/std/cell/struct.LazyCell.html
[`LazyLock`]: https://doc.rust-lang.org/std/sync/struct.LazyLock.html
[1.70]: https://blog.rust-lang.org/2023/06/01/Rust-1.70.0.html#oncecell-and-oncelock

[mut]: https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=b4a5948e58149068f583fb492079aefa

## 默认参数可能与捕获式闭包冲突

[E0121]: https://doc.rust-lang.org/stable/error_codes/E0121.html

`LazyLock` 在这个问题上的相关设计为：
* 携带一个函数指针默认参数：`struct LazyLock<T, F = fn() -> T> { ... }`，目的是让 `static LAZY: LazyLock<Type> = ...`
  语法生效。这样做是因为 [`_` 无法出现在 static 中][E0121]，而函数指针作为默认参数可以很好地被编译器推断而无需指定它。
  注意：[函数指针](https://doc.rust-lang.org/reference/types/function-pointer.html) 有两个来源，函数项和非捕获式闭包。

  ```rust
  mod not_work {
      // 无默认参数
      struct Lazy<T, F>(T, F);

      static L1: Lazy<()> = Lazy((), || {});
      static L2: Lazy<(), _> = Lazy((), || {});
  }
  mod work {
      // 默认参数
      struct Lazy<T, F = fn() -> T>(T, F);
      static L: Lazy<()> = Lazy((), || {});
  }

  fn main() {}
  ```

  ```rust
  // L1 遇到的问题
  error[E0107]: struct takes 2 generic arguments but 1 generic argument was supplied
   --> src/main.rs:3:16
    |
  3 |     static L1: Lazy<()> = Lazy((), || {});
    |                ^^^^ -- supplied 1 generic argument

    |                |
    |                expected 2 generic arguments
    |
  note: struct defined here, with 2 generic parameters: `T`, `F`

   --> src/main.rs:2:12
    |
  2 |     struct Lazy<T, F>(T, F);
    |            ^^^^ -  -
  help: add missing generic argument
    |
  3 |     static L1: Lazy<(), F> = Lazy((), || {});
    |                       +++

  // L2 遇到的问题
  error[E0121]: the placeholder `_` is not allowed within types on item signatures for static variables
   --> src/main.rs:2:11
    |
  2 | static L: Lazy<(), _> = Lazy((), || {});
    |           ^^^^^^^^^^^ not allowed in type signatures
  ```

* 在实现上，参数 `F` 采用 trait bound，即 `impl<T, F: FnOnce() -> T> LazyLock<T, F>`，这实际上允许函数指针和 **捕获式** 闭包、
  以及 `Box<dyn FnOnce>` 之类的类型作为 `F`。编译器可以推断闭包，但如果使用者以 `Lazy<Type>` 方式标注类型，会导致捕获式闭包与
  默认参数发生类型冲突，因为它们无法转化。

  ```rust
  #![feature(lazy_cell)]
  use std::sync::LazyLock as Lazy;

  fn main() {
      let env = "hello".to_string();

      let ok1 = Lazy::new(|| env);
      let ok2: Lazy<String, _> = Lazy::new(|| env);

      let err: Lazy<String> = Lazy::new(|| env);

  }
  ```

  在 `err` 那行会出现：


  ```rust
  error[E0308]: mismatched types
    --> src/main.rs:10:39

     |
  10 |     let err: Lazy<String> = Lazy::new(|| env);
     |                             --------- ^^^^^^ expected fn pointer, found closure
     |                             |

     |                             arguments to this function are incorrect
     |
     = note: expected fn pointer `fn() -> String`
                   found closure `[closure@src/main.rs:10:39: 10:41]`

  note: closures can only be coerced to `fn` types if they do not capture any variables
    --> src/main.rs:10:42
     |
  10 |     let err: Lazy<String> = Lazy::new(|| env);
     |                                          ^^^ `env` captured here
  note: associated function defined here
    --> /rustc/7bd81ee1902c049691d0a1f03be5558bee51d100/library/std/src/sync/lazy_lock.rs:68:18
  ```

## 需要处理协变

`once_cell` 的 `Lazy` 在 2021 年遇到一个 [协变问题](https://github.com/matklad/once_cell/issues/167)。

在那里没有人提出有问题的代码，所以我自己写了一个：

```rust
use once_cell::sync::Lazy as LazyLock;

type Lazy<'a, T> = LazyLock<T, Box<dyn FnOnce() -> T + 'a>>;
fn main() {
    let s = String::new();
    let f = || s.len();
    let l = Lazy::new(Box::new(f));
    {
        g(&l, &String::new()); // invariance
        h(&s, &String::new()); // covariance
    }
}

fn g<'a>(_lazy: &Lazy<'a, usize>, _: &'a str) {}
fn h<'a>(_: &'a str, _: &'a str) {}
```

这不会编译：

```rust
error[E0716]: temporary value dropped while borrowed
  --> src/main.rs:11:16

   |

11 |         g(&l, &String::new());

   |                ^^^^^^^^^^^^^ - temporary value is freed at the end of this statement
   |                |
   |                creates a temporary value which is freed while still in use
...
14 | }

   | - borrow might be used here, when `l` is dropped and runs the `Drop` code for type `LazyLock`
   |
   = note: consider using a `let` binding to create a longer lived value
```

为了看到麻烦所在，我特意标注了不变和协变两种情况：
* 对于函数 h，两个引用的生命周期都是协变的，所以一个更长的引用 `&s` 通过协变缩短成与临时引用 `&String::new()`
  一样的生命周期，这可以编译，而且工作地很好
* 但对于函数 g，`Lazy<'a, usize>` 是一个复合结构，`'a` 因为其背后的 `UnsafeCell` 而不变 (invariant)，
  这意味着一旦构造了 `Lazy`，那个生命周期无法合理地缩短！

```rust
#![feature(lazy_cell)]
use std::sync::LazyLock; // 当前 once_cell 和标准库都存在这个问题

type Lazy<'a, T> = LazyLock<T, Box<dyn FnOnce() -> T + 'a>>;
fn main() {
    let s = String::new();
    let f = || s.len();             // 'a 产生于闭包所借用的 &'a s 
    let l = Lazy::new(Box::new(f)); // Lazy<'a, usize> 对 'a 是不变的，即 'a 无法缩短
    {
        g(&l, &String::new());      // 函数 g 要求 &String::new() 活到 'a 那么长，这是不可能的
    }
}

fn g<'a>(_lazy: &Lazy<'a, usize>, _: &'a str) {}
```

解决这个问题的关键在于，`Lazy` 需要以协变的方式处理这个生命周期，而不是以不变的方式处理，这样就可以和
`h(&s, &String::new())` 一样编译了。

@danielhenrymantilla 在 [PR#230] 和 [PR#233] 中使用 `ManuallyDrop` 和 union 技巧处理了协变问题，但导致 dropck
问题，目前需要不稳定 `#[may_dangle]` 方式解决（或者更高深的适用于 stable Rust 的技巧），最终通过 [PR#236] 取消了实验性的提交。

[PR#230]: https://github.com/matklad/once_cell/pull/230
[PR#233]: https://github.com/matklad/once_cell/pull/233
[PR#236]: https://github.com/matklad/once_cell/pull/236
