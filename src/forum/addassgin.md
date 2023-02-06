# `+=` 运算符与 MIR 应用

> 本文 `+=` 运算符部分整理自 [Why does += require manual dereference when AddAssign() does not?][original-post] 后半部分，
> MIR 部分是我自己补充的。

[original-post]: https://users.rust-lang.org/t/why-does-require-manual-dereference-when-addassign-does-not/88028/35

## `+=` 解语法糖

一个基础，但很少会思考的问题，Rust 的 `+=` 运算符是什么代码的语法糖？

### `a = a + b` 不等价于 `a += b`

`a = a + b` 是 `a += b` 的语法糖吗？这意味着任何 `a += b` 与任何 `a = a + b` 代码等价。

如果以标准库定义的 impls 为例子，你可能觉得两种写法都能编译，而且结果一致。

但考虑以下自定义类型的实现：

```rust,editable
use std::ops::{Add, AddAssign};

fn main() {
    let mut s = S;
    s += (); // ok
    s = s + (); // error: expected struct `S`, found `()`
}

struct S;

impl Add<()> for S {
    type Output = ();
    fn add(self, _: ()) { }
}

impl AddAssign<()> for S {
    fn add_assign(&mut self, _: ()) { }
}
```

代码不通过，原因是显然的，`s + ()` 的类型是 `()`，无法赋值给 `s` —— `a = a + b` 不是 `a += b` 的语法糖。

从运算符的 trait 定义来看（以 `+` vs `+=` 为例），它们没有任何关系：

```rust,ignore
pub trait Add<Rhs = Self> {
    type Output;

    fn add(self, rhs: Rhs) -> Self::Output;
}

pub trait AddAssign<Rhs = Self> {
    fn add_assign(&mut self, rhs: Rhs);
}
```

### `AddAssign::add_assign(&mut a, b)` 与 `a += b` 

`+` 和 `+=` 是典型的二元运算符和复合赋值运算符。根据各自的运算符 trait 定义，可以得到以下解语法糖：

* `a + b` 实际调用 `Add::add(a, b)`
* `a += b` 实际调用 `AddAssign::add_assign(&mut a, b)`

注意以下几点：
* 若 a 和 b 拥有所有权时，其右侧运算数 b 的所有权被获取[^rhs-ownership]，而对待左侧运算数所有权的方式并不相同：
    * `a + b` 获取了 a 的所有权（无法再使用 a）
    * `a += b` 获取了 a 的独占引用，而非所有权（a 必须是 mut 的，而且此后仍可以使用 a）
* 若 a 或 b 不拥有所有权时，则不存在对 a 或 b 所有权的转移[^borrowing]：
    * 当 implementor 为引用时，参数一并没有发生所有权的移动
    * 当泛型类型参数为引用时，参数二并没有发生所有权的移动
* 调用的形式最好使用完全限定语法，而不是方法调用语法。这是因为方法调用表达式存在隐式的 [自动引用/解引用][auto-ref-deref]，而基于类型的分析才更可靠。
    * `a + b` 实际调用 `<TypeOfA as Add<TypeOfB>>::add(a, b)`，优于 `a.add(b)`
    * `a += b` 实际调用 `<TypeOfA as AddAssign<TypeOfB>>::add_assign(&mut a, b)`，优于 `(&mut a).add_assign(b)`

[^rhs-ownership]: 一个[例子](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=a38b2edef7d2c22bd7b7735ccb9161a7)

[^borrowing]: 另一个[例子](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=9267b4b9e4dbd95e4c69b50cb0bde6f3)

[auto-ref-deref]: https://doc.rust-lang.org/reference/expressions/method-call-expr.html

本文的重点在于 `a += b`，而不是 `a + b`，所以对 `a + b` 的内容就此结束。

对于分析 `a += b`，我遵循以下思考流程：
1. 写下两侧的类型，如 `Self += Rhs`
2. 实际调用的形式，如
    * `<Self as AddAssign<Rhs>>::add_assign(&mut Self, Rhs)`
    * `<Self as AddAssign<Rhs>>::add_assign(&mut a, b)`
3. 完全限定语法的几种等价形式：
    * `Self: AddAssign<Rhs>`：这在分析 trait bounds 时常用
    * `impl AddAssign<Rhs> for Self`：这在搜索具体实现时有用[^impl-search]

[^impl-search]: 拓展阅读：运用这套流程分析 `==` 操作符的具体的[例子](https://users.rust-lang.org/t/more-surprises-with-the-operator/87560/24)

但像 `+=` 这样的“复合赋值运算符”，一个鲜为人知的规则是关于两侧运算数的求值顺序。

## 赋值表达式的求值顺序

通过一个示例来感受求值顺序为什么重要：

```rust
*{
    print!("lhs ");
    &mut 0
} += {
    print!("rhs ");
    0
};

*{
    print!("lhs ");
    &mut String::from("a")
} += {
    print!("rhs ");
    "b"
};
```

这段代码打印什么？

如果你能准确说出和解释打印的内容，那么这小节内容可以跳过了。

如果你不知道答案，请往下看。

[example@eddyb]: https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=c2ad96fece1c4ee91295968742d65ec8
[example@tczajka]: https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=32448a960543f9f2a3acca34daada0b1

### 规则

Rust 中，[大部分表达式是从左往右求值的][LTR]，比如对于方法调用表达式 (method call expression) `a.add(b)`，脱糖为
`Add::add(a, b)`，然后先计算左边的 `a`，再计算右边的 `b`。但赋值表达式不一定是从左到右求值。

[LTR]: https://doc.rust-lang.org/reference/expressions.html#evaluation-order-of-operands

Rust 具有两种“赋值表达式”：
* 赋值表达式 ([assignment expressions])：将一个值移动进指定的地方，语法为 `assignee operand = assigned value operand`。
* 复合赋值表达式 ([compound assignment expressions])：将运算/逻辑二元运算符与赋值表达式结合起来，语法为
  `assigned operand 操作符 modifying operand`，其中“操作符”为一个标记后跟一个 `=`（中间不含空格），比如 `+=`、`|=`、`<<=`。

[assignment expressions]: https://doc.rust-lang.org/reference/expressions/operator-expr.html#assignment-expressions
[compound assignment expressions]: https://doc.rust-lang.org/reference/expressions/operator-expr.html#compound-assignment-expressions

两侧运算数的名称非常不直观，所以我使用左右两侧的表达方式来称呼它们。实际上，它们以前被称作“左值” (lvalue) 和“右值” (rvalue)。

对赋值表达式来说，先计算等号右侧的值，再计算等号左侧的值，即从右到左；对于解构赋值，其内部求值顺序为从左到右。

```rust
# let (mut a, mut b);

(a, b) = (3, 4); // 从右到左：先计算等号右侧的 (3, 4)，再赋值给等号左侧的 (a, b)

// 脱糖为

{
    let (_a, _b) = (3, 4); // 解构赋值过程中，从左到右
    a = _a; // 先赋值给解构模式左边的 a
    b = _b; // 再赋值给解构模式右边的 b
}
```

对于复合赋值表达式，若两侧的类型同时为 primitives，从右到左计算；否则从左到右计算。

回到本小节开头的示例，现在可以仔细分析代码了：

```rust
// 等号两侧的类型都为 `i32`，它是 primitive type，所以从右到左计算，打印 `rhs lhs `
*{
    print!("lhs ");
    &mut 0
} += {
    print!("rhs ");
    0
};

// 等号左右的类型为 `String` 和 `&str`，都不是 primitive type，所以从左到右计算，打印 `lhs rhs `
*{
    print!("lhs ");
    &mut String::from("a")
} += {
    print!("rhs ");
    "b"
};
```

或许这些细节你会感到困惑：

| 问                                       | 答                                                                                            |
|------------------------------------------|-----------------------------------------------------------------------------------------------|
| 等号左侧为什么要那样写？                 | 因为不允许直接写 [`0 += ...`][invalid-left]                                                   |
| 为什么左侧可以维持临时的引用 `&mut`？ | 见 [temporary-lifetime-extension]                                                             |
| 为什么左侧类型是 `i32`                   | `0` 的类型为 `i32`，这是 Rust 默认推断的；`&mut 0` 类型为 `&mut i32`；`*&mut 0` 类型为 `i32`  |
| 为什么 `i32` 是 primitive type？         | 见标准库 [primitive types]                                                                    |
| 什么是 primitive type？                  | 见标准库 [primitive types]                                                                    |
| 为什么 `&str` 不是 primitive type？      | 见标准库 [primitive types]，且见下面的例子：`i32` 是，`&i32` 不是，所以 `str` 是，`&str` 不是 |

[invalid-left]: https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=68ca5e7d3886f062fa03cdb1a850974a
[primitive types]: https://doc.rust-lang.org/std/index.html#primitives

[temporary-lifetime-extension]: https://doc.rust-lang.org/reference/destructors.html#temporary-lifetime-extension
[evaluation-order]: https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=37fb21d722c2012be3eec2df9d5a17bb
[eval-order.rs]: https://github.com/rust-lang/rust/blob/1.58.0/src/test/ui/expr/compound-assignment/eval-order.rs

总而言之，在 Rust 中，大部分表达式的求值顺序是从左往右的，仅有少数地方是从右往左的，比如：
1. 赋值表达式：先计算等号右侧
2. 复合赋值表达式：仅在两侧运算数都为 primitive types 时才先计算右侧运算数。为了巩固这一条，请确保你完全理解下面的
   [代码][evaluation-order] 和注释。此外，你还可以看懂 rustc 的这个 [测试代码][eval-order.rs]。

```rust
use std::num::Wrapping;

macro_rules! add_assign {
    ($e1:expr, $e2:expr) => {
        *({print!("lhs "); &mut $e1}) += {print!("rhs "); $e2};
        println!("");
    }
}

fn main() {
    add_assign!(1, 2); // rhs lhs: both operands are primitives
    add_assign!(1, &2); // lhs rhs: Rhs &i32 is not a direct primitive
    add_assign!(String::new(), ""); // lhs rhs: neither operands are primitives
    add_assign!(Wrapping(1), Wrapping(2)); // lhs rhs: neither operands are primitives
    // So usually the execution order of `+=` is LTR (left-to-right)
}
```

### MIR

Rust 的 MIR 是 HIR 到 LLVM IR 的中间产物，对 Rust 众多语法糖进行了脱糖，并且极大地精简了 Rust
语法（但并非其语法子集），是观察和分析 Rust 代码的常用手段，尤其是在控制流图和借用检查方面。

获取 MIR 的最简便的方式是通过 playground 左上角下拉框，点击 MIR 按钮。

此外，你还可以使用 `rustc src/main.rs -Z dump-mir=main` 或 `cargo rustc -- -Z dump-mir=main` 获得有关 main 函数完整的 MIR
* 查看 `mir_dump/main.main.-------.renumber.0.mir` 等文件
* 使用 `cargo rustc -- -Z help` 查看更多 mir 相关命令
* 相关 MIR 资料
    * [Rust Blog: Introducing MIR](https://blog.rust-lang.org/2016/04/19/MIR.html) 友好的官方入门解释
    * [rustc-dev-guide: MIR Debugging](https://rustc-dev-guide.rust-lang.org/mir/debugging.html)
    * [rustc-dev-guide: The MIR (Mid-level IR)](https://rustc-dev-guide.rust-lang.org/mir)

对于上一节开头的示例：

```rust
// 去除了无关和冗杂的 print!，将这段代码复制到 play.rust-lang.org 查看 MIR
*{ &mut 0 } += 0;
*{ &mut String::from("a") } += "b";
```

关键的 MIR 输出：

```rust,ignore
bb0: {
    _1 = const 0_i32;
    _3 = const 0_i32;
    _2 = &mut _3;
    _4 = CheckedAdd((*_2), _1);
    assert(!move (_4.1: bool), "attempt to compute `{} + {}`, which would overflow", (*_2), move _1) -> bb1;
}

bb2: {
    _7 = &mut _8;
    _6 = &mut (*_7);
    _10 = const "b";
    _9 = _10;
    _5 = <String as AddAssign<&str>>::add_assign(move _6, move _9) -> [return: bb3, unwind: bb5];
}
```

这很容易解释 `+=` 的语法脱糖和真正的执行顺序：

* `_4 = CheckedAdd((*_2), _1)` 这里的执行顺序是从右到左（注意观察编号），并且不是调用 `<i32 as AddAssign<i32>>::add_assign`，
  而是直接调用 `CheckedAdd` 函数。
  * 而 [`add_assign!(1, &2)`] 则对应 `_1 = <i32 as AddAssign<&i32>>::add_assign(move _2, move _13) -> bb5`，顺序从左到右，调用了重载的
    `+=` trait 方法。
* `_5 = <String as AddAssign<&str>>::add_assign(move _6, move _9)` 这里的顺序是从左到右，调用的是重载的 `+=` trait 方法。

[`add_assign!(1, &2)`]: https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=f4b583eef140c74def0f887da46746d9

## 单一实现下的强转

遵循前面我提到的流程，对于以下正常工作代码，第一步，写下左右两侧的类型，你会得到 `S += &&&&&&()`，实际不存在这个实现，因为
`S` 仅有 `S: AddAssign<&()>`。这发生了什么？

```rust
struct S;
impl std::ops::AddAssign<&()> for S {
    fn add_assign(&mut self, _: &()) {}
}

fn main() {
    let mut s = S;
    let rrrrrr = &&&&&&();
    s += rrrrrr;
}
```

通过 MIR，你会发现
* `<S as AddAssign<&()>>::add_assign(move _4, move _5)` 表明从左到右执行，因为两侧运算数不是 primitive type
* 传给 `add_assign` 的第二个参数，其类型并不是变量 `rrrrrr` 的类型 `&&&&&&()`，而是经过 5 次解引用之后的 `&()` 类型

```rust,ignore
bb0: {
    _6 = const _;
    _4 = &mut _1;
    _7 = deref_copy (*_2);
    _8 = deref_copy (*_7);
    _9 = deref_copy (*_8);
    _10 = deref_copy (*_9);
    _11 = deref_copy (*_10);
    _5 = _11;
    _3 = <S as AddAssign<&()>>::add_assign(move _4, move _5) -> bb1;
}
```

这里隐式的解引用是因为强转，而函数参数是能够发生 [强转的地方][coercion sites] 之一。

并且，依据这段 MIR（注意看从上到下的执行过程），我们知道，对于已知的 `add_assign` 实现，执行顺序先于强转发生。

而当 `S` 的 `AddAssign` 实现是多个，强转被阻止，你需要传入准确的类型的值：

```rust
struct S;
impl std::ops::AddAssign<()> for S {
    fn add_assign(&mut self, _: ()) {}
}
impl std::ops::AddAssign<&()> for S {
    fn add_assign(&mut self, _: &()) {}
}

fn main() {
    let mut s = S;
    let rrrrrr = &&&&&&();
    s += rrrrrr;
}

// error[E0277]: cannot add-assign `&&&&&&()` to `S`
//   --> src/main.rs:12:7
//    |
// 12 |     s += rrrrrr;
//    |       ^^ no implementation for `S += &&&&&&()`
//    |
//    = help: the trait `AddAssign<&&&&&&()>` is not implemented for `S`
//    = help: the following other types implement trait `AddAssign<Rhs>`:
//              <S as AddAssign<&()>>
//              <S as AddAssign<()>>
```


[coercion sites]: https://doc.rust-lang.org/reference/type-coercions.html#coercion-sites

## 两阶段借用的参与

以下代码能够运行：
* 由于两侧类型不是 primitive type， `add_assign` 从左到右执行
* 但已经使用 `&mut self` 的情况下，为什么能够同时执行带 `&self` 的方法？

```rust
struct S;
impl std::ops::AddAssign<()> for S {
    fn add_assign(&mut self, _: ()) {}
}
impl S {
    fn no_op(&self) {}
}

fn main() {
    let mut s = S;
    s += s.no_op();
}
```

通常对于初学者， `&mut` 会有两个更高级的主题：
* 重新借用 (reborrow)
    * open 状态的 [Reference issue][Reference-reborrow]、[RFC issue][RFC-reborrow]，在迁移到 Chalk 之前，不会正式描述 reborrow
    * 它大概是说：我们看见的 `&'a mut T`，实际被自动转化成更短的 `&'b mut T`，从而看起来 `&mut T` 一直可用。这也发生在
      `&T` 上面，但通常我们对 `&mut T` 的 reborrow 更敏感。
    * 这一是个在 1.0 之前就有的[概念][1.0reborrow]
    * UCG 可能会对 reborrow 做出[说明][UCG-reborrow]
    * 一个直觉上的[理解][intuition-reborrow]
* 两阶段借用 (two-phase borrows)
    * 它在 rustc dev guide 上的 [正式介绍][two-phase]
    * 它大概是说，某些情况下 `&mut T` 会划分成两个阶段进行使用：
        * 在 reservation 阶段：`&mut T` 像是 `&T` 那样，以允许多个 `&T` 同时存在
        * 在 activated 阶段：`&mut T` 以完全独占的方式使用
    * 某些情况指以下三种情况之一（上述链接对具体例子都有分析）：
        * 调用 receiver 为 `&mut self` 的方法（包括方法调用时的自动引用）：如 `vec.push(vec.len())`
        * 函数参数中的 `&mut T` reborrow：如 `std::mem::replace(r, vec![r.len()])`
        * 重载的复合赋值运算符中隐式的 `&mut T`：如本小节示例
    * 源代码中，任何显式的 `&mut` 和 `ref mut` 都不是两阶段借用

[Reference-reborrow]: https://github.com/rust-lang/reference/issues/788
[RFC-reborrow]: https://github.com/rust-lang/rfcs/pull/2364#issuecomment-375444971
[1.0reborrow]: https://github.com/nikomatsakis/babysteps/blob/master/babysteps/_posts/2013-11-20-parameter-coercion-in-rust.markdown?plain=1#L78
[UCG-reborrow]: https://github.com/rust-lang/unsafe-code-guidelines/blob/master/wip/stacked-borrows.md#reborrowing
[intuition-reborrow]: https://users.rust-lang.org/t/unifying-borrow-and-reborrow-conceptually-via-access

[two-phase]: https://rustc-dev-guide.rust-lang.org/borrow_check/two_phase_borrows.html

MIR 可以帮助你看到两阶段借用。

```rust,ignore
bb0: { // reservation 阶段
    _3 = &mut _1; // 两阶段借用的第三种前提：重载的复合赋值运算符中隐式的 `&mut T`
    _5 = &_1;     // `&mut T` 暂时被视为 `&T`，从而允许在此处使用 `&T`
    _4 = S::no_op(move _5) -> bb1;
}
bb1: { // activated 阶段
    _2 = <S as AddAssign<()>>::add_assign(move _3, move _4) -> bb2;
}
```

## 实战

例子源自 [#72199][intresting-issue] issue，@steffahn 做了很好的 [解释][steffahn-reply]，这里从 MIR 角度进行补充。

[intresting-issue-mir]: https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=29a80e38136aa1662a1ae98fc227cfc6
[intresting-issue]: https://github.com/rust-lang/rust/issues/72199
[steffahn-reply]: https://github.com/rust-lang/rust/issues/72199#issuecomment-1399948291

### `Vec<i32>` 的 `v[i] += v[j]`

```rust
fn main() {
    let mut v = Vec::from([0, 1]); // 为了让 MIR 精简，故意不使用 vec![0, 1]
    v[0] += v[1]; // 第一步：i32 += i32
}

// 两侧为 primitive types， RTL: <i32 as Add<i32>>::add_assign(&mut v[0], v[1])
// 1. 计算 v[1]：对它脱糖 `<Vec<i32> as Index<usize>>::index(&v, 1)` 得到 `&i32`，然后解引用得到 `i32` 
// 2. 计算 &mut v[0]：对它脱糖 `<Vec<i32> as IndexMut<usize>>::index_mut(&mut v, 0)` 得到 `&mut i32`
// 可以看到先使用了 `&v`，再使用了 `&mut v`，通过借用检查

// 仅列出 MIR 中的重点
// let mut _1: std::vec::Vec<i32>;
// bb1: {
//     _5 = &_1;
//     _4 = <Vec<i32> as Index<usize>>::index(move _5, const 1_usize) -> [return: bb2, unwind: bb6];
// }
// bb2: {
//     _3 = (*_4);
//     _7 = &mut _1;
//     _6 = <Vec<i32> as IndexMut<usize>>::index_mut(move _7, const 0_usize) -> [return: bb3, unwind: bb6];
// }
// bb3: {
//     _8 = CheckedAdd((*_6), _3);
//     assert(!move (_8.1: bool), "attempt to compute `{} + {}`, which would overflow", (*_6), move _3) -> [success: bb4, unwind: bb6];
// }
// bb4: {
//     (*_6) = move (_8.0: i32);
//     drop(_1) -> bb5;
// }
```

### `&mut [Custom]` 的 `v[i] += v[j]`

```rust
#[derive(Clone, Copy)]
struct MyNum(i32);

impl std::ops::AddAssign for MyNum {
    fn add_assign(&mut self, rhs: MyNum) {
        *self = MyNum(self.0 + rhs.0)
    }
}

fn main() {
    let mut b = vec![MyNum(0), MyNum(1)];
    let v = b.as_mut_slice();
    v[0] += v[1]; // MyNum += MyNum
}

// LTR: <MyNum as Add<MyNum>>::add_assign(&mut v[0], v[1])
// 1. 计算 &mut v[0]：获取和维持对第 0 元素的独占引用，但只进入 reservation 阶段，将 &mut 视为 &，从而继续使用切片
// 2. 计算 v[1]：在 `&mut v[0]` 的第一阶段，通过 `*_10` 和索引拷贝 MyNum
// 3. 调用方法，`&mut v[0]` 进入 activated 阶段

// 仅列出 MIR 中的重点
// let mut _1: std::vec::Vec<MyNum>;
// bb2: {
//     _11 = &mut _1;
//     _10 = Vec::<MyNum>::as_mut_slice(move _11) -> [return: bb3, unwind: bb8];
// }
// bb3: { // 索引前进行了边界检查
//     _14 = const 0_usize;
//     _15 = Len((*_10));
//     _16 = Lt(_14, _15);
//     assert(move _16, "index out of bounds: the length is {} but the index is {}", move _15, _14) -> [success: bb4, unwind: bb8];
// }
// bb4: {
//     _13 = &mut (*_10)[_14]; // 获取 &mut v[0]，进入 reservation 阶段
//     _18 = const 1_usize; // 索引前进行了边界检查
//     _19 = Len((*_10));
//     _20 = Lt(_18, _19);
//     assert(move _20, "index out of bounds: the length is {} but the index is {}", move _19, _18) -> [success: bb5, unwind: bb8];
// }
// bb5: {
//     _17 = (*_10)[_18]; // 计算 v[1]
//     _12 = <MyNum as AddAssign>::add_assign(move _13, move _17) -> [return: bb6, unwind: bb8]; // activated 阶段
// }
```

### `Vec<Custom>` 的 `v[i] += v[j]`

```rust
#[derive(Clone, Copy)]
struct MyNum(i32);

impl std::ops::AddAssign for MyNum {
    fn add_assign(&mut self, rhs: MyNum) {
        *self = MyNum(self.0 + rhs.0)
    }
}

fn main() {
    let mut b = vec![MyNum(0), MyNum(1)];
    b[0] += b[1];
}
```

它无法编译成功，但编译器提示你怎么 [解决][fix-vec-addassign]（把右侧的值赋给局部变量，然后使用该变量）：

[fix-vec-addassign]: https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=f093f2c06351af63fa11cdf80714c8c4

```rust,ignore
error[E0502]: cannot borrow `b` as immutable because it is also borrowed as mutable
  --> src/main.rs:12:13
   |
12 |     b[0] += b[1];
   |     --------^---
   |     |       |
   |     |       immutable borrow occurs here
   |     mutable borrow occurs here
   |     mutable borrow later used here
   |
help: try adding a local storing this...
  --> src/main.rs:12:13
   |
12 |     b[0] += b[1];
   |             ^^^^
help: ...and then using that local here
  --> src/main.rs:12:5
   |
12 |     b[0] += b[1];
   |     ^^^^^^^^^^^^
```

当你试着从 MIR 分析为什么这样，你会发现 playground 因为编译失败而没有 MIR 的结果，提示为
`Unable to locate file for Rust MIR output`。

此时，你仍可以在本地获取一部分 MIR 结果，因为 MIR 其实经过许多次迭代，`mir_dump` 文件夹下保留了半成品：运行
`cargo rustc --  -Z dump-mir=main`，查看 `mir_dump/simd.main.-------.renumber.0.mir` 文件。

```rust,ignore
// 仅列出关键部分
bb4: {
    _13 = &mut _1;
    _12 = <Vec<MyNum> as IndexMut<usize>>::index_mut(move _13, const 0_usize) -> [return: bb5, unwind: bb9];
}
bb5: {
    _11 = &mut (*_12);
    StorageDead(_13);
    StorageLive(_14);
    StorageLive(_15);
    StorageLive(_16);
    _16 = &_1;
    _15 = <Vec<MyNum> as Index<usize>>::index(move _16, const 1_usize) -> [return: bb6, unwind: bb9];
}
bb6: {
    _14 = (*_15);
    StorageDead(_16);
    _10 = <MyNum as AddAssign>::add_assign(move _11, move _14) -> [return: bb7, unwind: bb9];
}
```

把它与上一小节在 `&mut [MyNum]` 的 MIR 进行对比，你会发现在 `&mut Vec<MyNum>` 上没有发生两阶段借用：
* 观察两个 MIR 片段的 `move _13`，第二个片段的 `&mut _1` 借用已经在获取索引时结束（未能到达 `add_assign`），而第一个在调用 `add_assign` 时结束
* 所以 `Vec<MyNum>` 上的 `b[0] += b[1]` 是通过两个不同的 `&mut Vec<MyNum>` 和 `&Vec<MyNum>`，分别得到 `&mut MyNum` 和 `MyNum` 两个操作数

而 `_10 = <MyNum as AddAssign>::add_assign(move _11, move _14)` 延长了 `_11` 的生命周期（从而延长 `_12`、`_13`、最终 `&mut _1` 的生命周期），导致与
`&_1` 生命周期交叉。

```rust,ignore
// b[0] += b[1] on &mut [MyNum]

_10 = Vec::<MyNum>::as_mut_slice(move _11) // _10: &mut [MyNum]

_13 = &mut (*_10)[_14]; // two-phase
_17 = (*_10)[_18];      // reservation 阶段

_12 = <MyNum as AddAssign>::add_assign(move _13, move _17) // activated 阶段

// b[0] += b[1] on Vec<MyNum>

_13 = &mut _1; // _1: Vec<MyNum>
_12 = <Vec<MyNum> as IndexMut<usize>>::index_mut(move _13, const 0_usize) // _13: &mut Vec<MyNum>, _12: &mut MyNum
_11 = &mut (*_12); // reborrow, _11: &mut MyNum

_16 = &_1;
_15 = <Vec<MyNum> as Index<usize>>::index(move _16, const 1_usize) // _16: &Vec<MyNum>, _15: &mut MyNum
_14 = (*_15);

_10 = <MyNum as AddAssign>::add_assign(move _11, move _14)
```

## 总结

* `+=` 是可重载的复合赋值运算符，`Self += Rhs` 脱糖为 `<Self as AddAssign<Rhs>>::add_assign(&mut Self, Rhs)`，但
    * 对两侧为 primitive types 的运算数，先计算 `Rhs`，再计算 `Self`，然后调用编译器实现的相加函数
    * 若至少有一侧运算数不为 primitive types，则先计算 `Self`，再计算 `Rhs`，然后调用重载后的实现（即 `<Self as AddAssign<Rhs>>::add_assign`）
* 大多数表达式是从左到右执行的。从右到左是特殊情况，比如
    * 赋值表达式中，先计算 `=` 右侧的值，再计算左侧
    * 复合赋值表达式中，两侧为 primitive types 的运算数时，先计算复合赋值运算符右侧，再计算左侧
* MIR 是 Rust 编译过程的重要一环，（无论在代码编译成功还是失败的情况下）也可以成为辅助你分析的 Rust 代码的工具


