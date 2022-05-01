> 本文内容整理自：[https://users.rust-lang.org/t/generic-referencing-enum-inner-data/66342](https://users.rust-lang.org/t/generic-referencing-enum-inner-data/66342)
>

# 同质形式的 variants

如果你定义一个这样的枚举体：

```RUST
#[derive(Debug)]
enum Foo {
    Bar(u32),
    Bink(u32),
}
```

这是一种 [data-carrying](https://rust-lang.github.io/unsafe-code-guidelines/layout/enums.html) 的枚举体，而且它特殊在：

1. variants 中携带你所关心的 `u32` 类型的数据；
2. 这个类型位于 tuple variant 在第一个位置上。

从而它们 (variants)  形式上同质：相同的类型位于相同的位置。

那么，为了获取到 `u32` 数据，通常我们采用模式匹配的方式取出数据，而且对于枚举体，最常用 `match` 匹配：

```RUST
// src: https://users.rust-lang.org/t/generic-referencing-enum-inner-data/66342/2
impl Foo {
    fn get_value(&self) -> u32 {
        match self {
            Bar(value) => *value,
            Bink(value) => *value,
        }
    }
}
```

而 [@CKalt](https://users.rust-lang.org/u/CKalt) 提出一个很有启发性的观点：

> It would be nice, if, when all the variants of an enum share an identical type, that there were a way to refer to that data by it's position instead of having to pull it would with some long match statement, etc...
>
> 当所有 variants 都有相同的类型时，是否可能有一种方式，按照位置引用/访问数据，而不是只能写很长的匹配语句。
>

习惯上把 variant 类比 struct，因为它们都有：常规型、元组型、unit 型。

而元组结构体可以直接通过 `.number_of_position` 语法访问数据，那么在同质形式的 variants 中也使用这种语法，是不是更方便呢：

```RUST
let bink = Foo::Bink(20);
println!("bink num = {}", bink.0);
```

当然，这种同质形式的 variants 很少见，因为很多时候，一个枚举体里的各个 variant 形态各异：可能携带数据也可能不携带数据，而且即便携带数据，其数量、类型、位置都不一定相同。

可是这个问题并不算无趣，将思维拓展之后，会发现这是一个值得学习的案例（当然不仅仅是学习 variant）。

# 思路一：重新设计数据结构/数据类型

回到那个例子：你最终需要 `u32` 类型，那么你可以考虑从一开始就把这个数据从枚举体中分离出来。

```RUST
// src: https://users.rust-lang.org/t/generic-referencing-enum-inner-data/66342/2
struct Foo {
    value: u32,
    kind: FooKind,
}

enum FooKind {
    Bar,
    Bink,
}
```

虽然你依然需要使用 `match` 匹配每种情况，但是你的数据和流程逻辑都放在结构体类型上——枚举体仅仅是作为一个标签而已（`FooKind` 不携带数据，被称作 fieldless enum）。这样做的缺点可能在于：因为结构体对齐方式，其内存布局会不太紧凑。

# 思路二：善用模式匹配

在 Rust 1.53 版本中，模式匹配有了一种新模式叫：[or-patterns](https://blog.rust-lang.org/2021/06/17/Rust-1.53.0.html#or-patterns)，其一般阐述见 [reference#or-patterns](https://doc.rust-lang.org/nightly/reference/patterns.html#or-patterns) 。

1. 当所有 variants 都是元组型，而且第一个位置都是相同类型，那么你可以这样写：

```rust,editable
// src: https://users.rust-lang.org/t/generic-referencing-enum-inner-data/66342/6
enum Foo {
    Bar(u32, u32, f64),
    Bink(u32, f64),
}

fn demo(x: Foo) {
    use Foo::*;
    let (Bar(num, ..) | Bink(num, _)) = x;
    println!("{}", num);
}

fn main() {
    demo(Foo::Bink(0, 1.));
}
```

2. 类似地，当所有 variants 都是常规结构体型，而且相同类型的字段名一致，那么你可以这样写：

```rust,editable
enum Foo {
    Bar  {x: u32, y: u32, z: f64},
    Bink {x: u32, y: f64},
}

fn demo(foo: Foo) {
    use Foo::*;
    // 这个 `()` 叫分组模式 (Grouped Pattern) 用于显式控制复合模式的优先级
    // 在多种模式表达不清的时候，用它来明确优先级
    let (Bar {x, ..} | Bink {x, ..}) = foo; 
    println!("{}", x);
}

fn main() {
    demo(Foo::Bink {x: 0, y: 1.});
}
```

即便相同类型的字段名不一致，你依然可以利用强大的模式匹配： 

```rust,editable
enum Foo {
    Bar {xx: u32, y: u32, z: f64},
    Bink{x: u32, y: f64},
}

fn demo(foo: Foo) {
    use Foo::*;
    let (Bar {xx: x, z, ..} | Bink {x, y: z, ..}) = foo;
    println!("{} {}", x, z);
}

fn main() {
    demo(Foo::Bink {x: 0, y: 1.2});
}
```


# 思路三：用宏来简化代码

当你以同一种方式重复编写代码时，可以考虑使用宏来简化。编写宏需要很多技巧，所以更需要通过经常学习和训练来提高。（“宏”有时候是声明宏和过程宏的统称，有时候专指声明宏，这里专指声明宏）

从 [@steffahn](https://users.rust-lang.org/u/steffahn) 的[回答](https://users.rust-lang.org/t/generic-referencing-enum-inner-data/66342/9)中，一起学习这些技巧吧。

## 技巧 1：`$name:pat`

在宏的 13 种分类片段符 (Fragment Specifiers) 里，专门有一种来匹配模式：[pat](https://zjp-cn.github.io/tlborm/macros/minutiae/fragment-specifiers.html#pat) 。在思路二中，我们一直在谈论模式匹配，所以这是引导我们考虑编写宏的起点。

从最简单的例子出发：

```RUST
#[derive(Debug)]
enum Foo {
    Bar(u32),
    Bink(u32),
}
```

 把 `(Bar(n) | Bink(n))` 部分用宏进行替换：

```rust,editable
//! src: https://users.rust-lang.org/t/generic-referencing-enum-inner-data/66342/9
//! author: https://users.rust-lang.org/u/steffahn
macro_rules! Foo {
    ($p:pat) => {
        Foo::Bar($p) | Foo::Bink($p)
    }
}

fn main() {
    let x = Foo::Bar(42);
    let Foo!(n) = x; // 即 let (Bar(n) | Bink(n)) = x;
    println!("{}", n);
}
```

由思路二可以知道，当 variants 是元组长度不一时或者常规结构体型时，我们都可以使用模式匹配语法轻松处理掉。

```rust,editable
//! src: https://users.rust-lang.org/t/generic-referencing-enum-inner-data/66342/9
//! author: https://users.rust-lang.org/u/steffahn
#[derive(Debug)]
enum Baz {
    Variant1 {
        x: u32,
        y: String,
        z: bool,
    },
    Variant2 {
        x: u32,
        z: bool,
        qux: fn(),
    }
}
 
macro_rules! Baz {
    ($($field:ident $(: $p:pat)?,)* ..) => {
        Baz::Variant1{ $($field $(: $p)?,)* .. } | Baz::Variant2{ $($field $(: $p)?,)* .. }
    }
}

fn main() {
    let baz = Baz::Variant1 { x: 42, y: "hello".into(), z: false };
    let Baz!{ x, z, .. } = &baz;
    println!("{:?} with `x` being {:?} and `z` being {:?}", baz, x, z);
}
```

这里有一些细节：

1. 所有 [item](https://doc.rust-lang.org/nightly/reference/items.html) 的声明顺序与使用顺序无关；但宏不是 item，宏必须在使用之前声明。
2. `Foo!` 和 `Baz!` 可适用于以下两种模式：

    1. 解构 ([destructuring](https://doc.rust-lang.org/nightly/reference/patterns.html#destructuring))：把值分解成其组成部分，所以 `let Foo!(n) = x;` 让 `n` 为 `u32` 类型。
    2. 以 `ref` 方式绑定：以非引用模式匹配引用时，绑定方式 ([binding mode](https://doc.rust-lang.org/nightly/reference/patterns.html#binding-modes)) 会自动变为  `ref` 或 `ref mut`（详细说明见 [RFC: match ergonomics](https://rust-lang.github.io/rfcs/2005-match-ergonomics.html)）。所以 `let Baz!{ x, z, .. } = &baz;` 中的 `x` 和 `z` 分别是 `&u32` 和 `&bool` 类型。
3. 正如 Rust By Example 的 [macros](https://doc.rust-lang.org/rust-by-example/macros.html) 一章提到的宏的优点：减少重复；自定义语法；可变长度的参数。虽然无法拥有 `Foo::Bink(20).0` 或者 `Baz::Variant1 { x: 42, y: "hello".into(), z: false }.x` 直接访问的语法，但我们依然可以通过宏和模式进行创造。

## 技巧 2：用宏来生成宏

当有很多类似 `Baz` 的枚举体时，我们可以用宏来生成同名宏 `Baz!`。

```rust,editable
//! src: https://users.rust-lang.org/t/generic-referencing-enum-inner-data/66342/9
//! author: https://users.rust-lang.org/u/steffahn
#![allow(unused)]

macro_rules! define_enum_macro {
    ($Type:ident, $($variant:ident),+ $(,)?) => {
        define_enum_macro!{#internal, [$], $Type, $($variant),+}
    };
    (#internal, [$dollar:tt], $Type:ident, $($variant:ident),+) => {
        macro_rules! $Type {
            ($dollar($field:ident $dollar(: $p:pat)?,)* ..) => {
                $($Type::$variant { $dollar($field $dollar(: $p)?,)* .. } )|+
            }
        }
    };
}

// -------------------------------------------------------------------------- //

#[derive(Debug)]
enum Baz {
    Variant1 {
        x: u32,
        y: String,
        z: bool,
    },
    Variant2 {
        x: u32,
        z: bool,
        qux: fn(),
    }
}

define_enum_macro!(Baz, Variant1, Variant2);

fn main() {
    let baz = Baz::Variant1 { x: 42, y: "hello".into(), z: false };
    let Baz!{ z, .. } = &baz;
    println!("{:?} with `z` being {:?}", x, z);
}
```

一些理解/解释：

1. 这里编写了两条解析规则：第一次解析的目的是匹配用户输入的语法，取出所需的 fragment specifiers；第二次解析后真正生成宏。
2. 这里编写了一条内用规则 ([internal rules](https://zjp-cn.github.io/tlborm/patterns/internal-rules.html))，主要原因是：生成宏需要 `$` 符号，而它无法直接在 transcriber（指 `=>` 之后的部分）表示出来，因此通过递归，把 `$` 符号作为 [tt](https://zjp-cn.github.io/tlborm/macros/minutiae/fragment-specifiers.html#tt) 分类符（用于匹配 tokens tree） 传给下一次解析。而 `#internal` 被视为内用规则的名称，你可以取任何名字，也可以使用其他前缀符号（比如常见的 `@internal_name`）。

---

结语：

1. 通过本篇文章，你可以感受到 Rust 的模式 ([patterns](https://doc.rust-lang.org/nightly/reference/patterns.html)) 无处不在，见识了它与宏完美结合的典范。
2. 如果你觉得这篇文章有用，请给 steffahn 的 [回答](https://users.rust-lang.org/t/generic-referencing-enum-inner-data/66342/9) 一个点赞以示鼓励。我只是对一则 Rust User Forum 的帖子进行了梳理、记录和补充，没有 steffahn 的回答，也就没有这篇文章。
3. [Rust User Forum](https://users.rust-lang.org/) 是学习 Rust 和体验 Rust 圈氛围的好地方，每次逛都可以学到许多额外的技巧与收获。一些高质量的回答淹没在茫茫帖子中，这是可惜的，或许以文章/笔记的方式，让更多人看见，这会是一种不错的尝试。
