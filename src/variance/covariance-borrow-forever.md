# 当 `&'a Type<'a>` 变成了永远借用

对生命周期有足够了结的 Rustancean 不会对以下代码感到不解。

```rust
struct Person<'a> {
    name: &'a str,
}

impl<'a> Person<'a> {
    //        vvvvvvvv `&'a Person<'a>`
    fn borrow(&'a self) -> &'a str {
        self.name
    }

    //                vvvvvvvvvvvv `&'a mut Person<'a>`
    fn borrow_forever(&'a mut self) -> &'a str {
        self.name
    }
}

fn fails(mut person: Person<'_>) {
    person.borrow_forever();
}// error: `person` dropped here while still borrowed

fn works(mut person: Person<'_>) {
    let _one = person.borrow();
    let _two = person.borrow();
    &mut person; // ok
}
```

我会简单略过这段代码不通过/通过的原因，如果你熟悉它们，则可以跳过以下两个小节（也无需关注上面的代码），而是从
[当 &'a Ty<'a> 牵绊你的时候](#当-a-tya-牵绊你的时候) 开始进入本文的正题。

# `&'a mut Ty<'a>` 是一种反模式

你绝不应该写 `&'a mut Ty<'a>`，因为它代表永远借用自己 —— 被借用的对象活 `'a` 那么长，而你指定了借用必须活 `'a`。

每当看到类似这种标注，都应该警惕这基本上是一个死胡同。因为一旦你获得 `&'a mut Ty<'a>`，那么面临两种选择
1. 一直使用它，但它是 `Ty<'a>` 的全周期独占引用， `Ty<'a>` 活多长，这个独占引用就维持多长，借用规则让你永远不允许 
   存在另一个独占引用 (`&'another mut Ty<'a>`)，也永远不允许有其他共享引用 (`&'another Ty<'a>`)。  
  （但允许 reborrows (`&'sub mut Ty<'a>`)，因为 `&'a mut` 中，`&mut` 对 `'a` 协变，见下面的 [永久借用中的协变]）
2. 不再使用它，那么 `&'a mut Ty<'a>` 的 `'a` 不再存活，也就是 `Ty<'a>` 不再存活，即无法再使用 `Ty<'a>`。

所以，`&'a mut Ty<'a>` 这个独占引用变成对 `Ty<'a>` 的永久独占访问，似乎 `Ty<'a>` 的所有权也随着这个独占引用被夺走了：
你永远无法得到 `Ty<'a>` 的所有权，也永远无法转移 `Ty<'a>` 的所有权 —— 因为在 Rust 中移动一个值的前提是，这个值不被借用。

你还可以读读以下链接，通过具体代码去理解：

* [自引用与生命周期](https://zjp-cn.github.io/translation/lifetime/self-referential.html#%E4%B8%80%E7%A7%8D%E5%8F%AF%E8%A1%8C%E4%BD%86%E6%97%A0%E7%94%A8%E5%81%9A%E6%B3%95)
* [Borrowing something forever](https://quinedot.github.io/rust-learning/pf-borrow-forever.html)

# `&'a Ty<'a>` 通常不会牵绊住你

这得益于 covariance（协变），也就是生命周期可以缩短的能力。具体来说，因为 `&'a T` 具有两处协变：
* `&` 对 `T` 是协变的，即 `&T` 可以当作 `&U` 去使用，只要 `T` 是 `U` 的子类型
* `&` 对 `'a` 是协变的，即 `&'a` 可以当作 `&'b` 去使用，只要 `'a: 'b`[^outlive]（`'a` 是 `'b` 的子类型）

所以**如果 `Ty<'a>` 对 `'a` 也是协变的话**[^subtype]， `&'a Ty<'a>` 可以先对 `Ty` 缩短成 `&'a Ty<'b>`，然后对 `'a`
缩短成 `&'b Ty<'b>`，从而每次使用 `&'a Ty<'a>`，都变成了临时的借用 `&'b Ty<'b>`，最终避免了永远借用 `'a`。

[^outlive]: `'a: 'b` 指 `'a` outlives `'b`，也就是 `'a` 至少和 `'b` 一样长，也就是 `'a` 活得和 `'b` 一样或者更长

[^subtype]: 对于上述 `'a: 'b`，有`Ty<'a>: Ty<'b>`，即 `Ty<'a>` 是 `Ty<'b>` 的子类型。注意：严格来说，对生命周期使用
`:` 记号是符合 Rust 的，但对类型使用 `:` 记号（`T: U`），是不太规范的。

如果你对 Rust 中的 subtyping 和 variance 不熟悉，请阅读：
* [Nomicon: subtyping](https://doc.rust-lang.org/nomicon/subtyping.html)
* [我的笔记](https://zjp-cn.github.io/rust-note/subtyping.html)


# 当 `&'a Ty<'a>` 牵绊你的时候

可以通过破坏 `&'a Ty<'a>` 进行协变的前提，来让 `&'a Ty<'a>` 绊倒你。

具体来说，如果 `Ty<'a>` 对 `'a` 不再是协变，而是不变 (invariant)，那么对于 `'a: 'b`
* `&'a Ty<'a>` 依然可以缩短成 `&'b Ty<'a>` （但仅限在 `'a` 存活期间，见 [永久借用中的协变]）
* `Ty<'a>` 无法缩短成 `Ty<'b>`，从而 `&'a Ty<'a>` 和 `&'b Ty<'a>` 无法缩短成 `&'b Ty<'b>`

实际上，`&'a Ty<'a>` 和 `&'a mut Ty<'a>` 其实几乎一模一样
([playground for `&'a mut Ty<'a>`](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=3b2db91b62a3cadd03c4b4d390b5bf03))
，唯一的区别在于一个是永久共享借用，另一个是永久独占借用。

```rust,editable
fn main() {
    let val = ();
    let mut ref_val = &val;
    let mut invariant = Invariant(&mut ref_val);
    invariant.borrow(); // Invariant<'a>::borrow(&'a Self)
    invariant.borrow(); // ok: 你总是可以有多个 &'a T

    // 但你不能做以下事情中的任何一件
    
    // 不能有 &'a mut T：因为 &'a mut T 和 &'a T 不能同时存在
    &mut invariant; // error: cannot borrow `invariant` as mutable because it is also borrowed as immutable

    // 没法 move：因为 move 一个变量的前提是这个变量不被借用
    let _move = invariant; // error: cannot move out of `invariant` because it is borrowed

    // 没法显式调用 drop（和按值方式接收参数的函数）：理由同“没法 move”
    invariant.consume();
    drop(invariant); // error: cannot move out of `invariant` because it is borrowed
}

struct Invariant<'a>(*mut &'a ()); // `*mut T` 中，`*mut` 对 T 不变
impl<'a> Invariant<'a> {
    fn borrow(&'a self) {}
    fn consume(self) {}
}
```

[永久借用中的协变]: #永久借用中的协变

# 永久借用中的协变

引用的生命周期是协变的：对于 `'a: 'b`，任何 `&'a` 或 `&'a mut`
都可以因为协变相应地缩短成 `&'b` 或 `&'b mut`。

生命周期是协变的（即生命周期可以缩短），而引用通常可以再借 
(reborrow)，这表明：一个长的生命周期，可以在它存活的状态中，被“分割”成互不相交的子生命周期。

这也适用于永久借用。以下两个代码展示了如何在永久借用的存活期间再借（重点在 `borrow` 内部）。

```rust
fn main() {
    let val = ();
    let mut ref_val = &val;
    let mut invariant = Invariant(&mut ref_val);
    invariant.borrow(); // Invariant<'a>::borrow(&'a mut Self)
    
    // error: cannot borrow `invariant` as mutable more than once at a time
    // 因为形成新的 &'another mut Ty<'a>，需要结束任何其他引用，而 &'a mut Ty<'a> 与 Ty<'a> 同生共死
    // invariant.temp_borrow(); 
}

struct Invariant<'a>(*mut &'a ()); // `*mut T` 中，`*mut` 对 T 不变
impl<'a> Invariant<'a> {
    fn borrow(&'a mut self) { // ok
        // 永久借用期间，进行多次 reborrows：&'temp (*(&'a mut self))
        // 长的生命周期被“分割”彼此成互不相交的子生命周期
        self.temp_borrow();
        self.temp_borrow();
        self.temp_borrow();
    }
    fn temp_borrow(&mut self) {}
    fn consume(self) {}
}
```

```rust
fn main() {
    let val = ();
    let mut ref_val = &val;
    let mut invariant = Invariant(&mut ref_val);
    invariant.borrow(); // Invariant<'a>::borrow(&'a Self)
    
    invariant.temp_borrow(); // ok: 共享借用可以共享同一个生命周期
    
    // error: cannot borrow `invariant` as mutable because it is also borrowed as immutable
    // 因为形成新的 &'another mut Ty<'a>，需要结束任何其他引用，而 &'a Ty<'a> 与 Ty<'a> 同生共死
    // invariant.temp_mut_borrow(); 
}

struct Invariant<'a>(*mut &'a ()); // `*mut T` 中，`*mut` 对 T 不变
impl<'a> Invariant<'a> {
    fn borrow(&'a self) { // ok
        // 思路一：
        // 永久借用期间，进行多次 reborrows：&'temp (*(&'a self))
        // 长的生命周期被“分割”彼此成互不相交的子生命周期 &'temp1、&'temp2、&'temp3
        
        // 思路二：共享借用可以共享同一个生命周期，以下都是 &'a self
        
        self.temp_borrow();
        self.temp_borrow();
        self.temp_borrow();
    }
    fn temp_borrow(&self) {}
    fn temp_mut_borrow(&mut self) {}
    fn consume(self) {}
}
```

# 附录：永远借用会如何绊住你的脚

## 与 drop check 交互

```rust
struct Invariant<'a>(*mut &'a ()); // 自身及其内部无需 Drop 
impl<'a> Invariant<'a> { fn borrow(&'a self) {} }

// 在函数内创建无 Drop 的 Invariant 并永远借用
// （这可能是你写 Rust 的第一步，代码示例成功编译）
fn ok() {
    let val = ();
    let mut ref_val = &val;
    let mut invariant = Invariant(&mut ref_val);

    invariant.borrow();
}
// 你以为这样就没问题？看下面 fail 的情况

// （你想对一段代码进行封装，却发现无法编译）
// 将所有权移入函数（在函数外创建 Invariant），并在函数内永远借用
// error: `val` does not live long enough
fn fail(val: Invariant<'_>) {
    val.borrow();
} // `val` dropped here while still borrowed
```

<details>
  <summary>当 Invariant 自身或者内部需要 Drop 时，原本无 Drop 时能通过的代码，现在无法通过。</summary>

```rust
// 原本无 Drop 时能通过的代码，现在无法通过
// error: `invariant` does not live long enough
fn fail() {
    let val = ();
    let mut ref_val = &val;
    let mut invariant = Invariant(&mut ref_val);

    invariant.borrow();
} // `invariant` dropped here while still borrowed
// borrow might be used here, when `invariant` is dropped and
// runs the `Drop` code for type `Invariant` 

struct Invariant<'a>(*mut &'a ()); 
impl<'a> Invariant<'a> { fn borrow(&'a self) {} }

// 当 Invariant 自身需要 Drop
impl Drop for Invariant<'_> { fn drop(&mut self) {} }
```

```rust
// 原本无 Drop 时能通过的代码，现在无法通过
// error: `invariant` does not live long enough
fn fail() {
    let val = ();
    let mut ref_val = &val;
    let invariant = Invariant(Inner(&mut ref_val));
    invariant.borrow();
} // 同 `当 Invariant 自身需要 Drop`

struct Invariant<'a>(Inner<'a>);
impl<'a> Invariant<'a> {
    fn borrow(&'a self) {}
}

// 当 Invariant 内部需要 Drop
struct Inner<'a>(*mut &'a ());
impl Drop for Inner<'_> { fn drop(&mut self) {} }
```

对这些情况的解释见 [Nomicon: drop check](https://doc.rust-lang.org/nomicon/dropck.html)。
</details>


## 与生命周期标注交互

有时，你的代码没有出现显式的 `&'a Ty<'a>`，但依然有可能因为生命周期标注，让你隐式得到它。

正如前述所言，`&'a Ty<'a>` 在 `Ty<'a>` 对 `'a` 协变时，通常不会造成影响；但若对 `'a` 不变，
`&'a Ty<'a>` 与 `&'a mut Ty<'a>` 几乎会造成同样的麻烦（唯一区别在于，一个是永久共享借用，另一个是永久独占借用）。

永久借用意味着
* `Ty<'a>` 与这个永久借用生死与共：`Ty<'a>` 和`&'a {mut} Ty<'a>` 要么一起存活，要么一起死亡
* `Ty<'a>` 这个值一直被借用：`Ty<'a>` 的所有权无法被获得和转移

在下述例子中，代码没有显式的 `&'a Ty<'a>` 或 `&'a mut Ty<'a>` （严格来说，其实存在 `&'a mut Ty<'a>`，因为
`&'a mut dyn std::fmt::Debug` 是 `&'a mut dyn ('a + std::fmt::Debug)` 的语法糖，但在这不是重点）。

```rust,editable
use std::cell::RefCell;
fn main() {
    let mut s1 = String::from("");
    let mut ss = &mut s1;
    let mut x = MyData(RefCell::new(&mut ss));
    let y = f(&x, &x);
    g(y, &x);

    &x; // ok
    // &mut x; // error: cannot borrow `x` as mutable because it is also borrowed as immutable
    // drop(x);// error: cannot move out of `x` because it is borrowed
}

struct MyData<'a>(RefCell<&'a mut dyn std::fmt::Debug>);

fn static_data<'any>() -> MyData<'any> {
    MyData(RefCell::new(Box::leak(Box::new(""))))
}
fn f<'a, 'b>(_: &'b MyData<'a>, _: &'b MyData<'a>) -> MyData<'b> {
    static_data()
}
fn g<'a, 'b>(_: MyData<'a>, _: &'b MyData<'a>) -> MyData<'b> {
    static_data()
}
```

但实际上里面存在一个隐式的 `&'a Ty<'a>`，且 `Ty<'a>` 对 `'a` 不变，从而遇到了与
[“当 &'a Ty<'a> 牵绊你的时候”](#当-a-tya-牵绊你的时候) 相同麻烦。

我自己推导生命周期会遵循一个套路，而第一步就是脱糖。f 和 g 两个函数的脱糖形式我已经写出来了，但它们的原型
[在这](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=497b781639a77c058ec2075fa4568b0f)，来自这个
[帖子](https://rustcc.cn/article?id=431b5cac-51db-470e-ba17-3f29344eb672)。（我知道帖子给的代码不是 Rust
惯用的代码，到处滥用了重置运算符和内存泄露，但这里的重点在于生命周期与型变，只需要看签名）

核心要点是每个方法调用变成最纯粹的形式，生命周期关系写得越清楚越好。我不会在这里描述具体怎么脱糖，这不是重点。

方法脱糖成函数也仅仅是个开始，接下来需要精简核心问题的代码。上面的代码已经是最能复现问题的精简版，具体过程也不是重点。

然后，一个核心步骤是，机械地写下源代码里每处相关的生命周期和类型，这基于你对生命周期的了解程度。

```rust,editable
use std::cell::RefCell;
fn main() {
    let mut s1 = String::from("");
    let mut ss = &mut s1; // ss: &'0 mut String

    // ss => &'0 mut String => &'1 mut String (协变, '1 来自 &'1 mut ss)
    // &'1 mut ss => &'1 mut &'1 mut String => &'1 mut dyn ('1 + Debug)
    // x: MyData(RefCell<&'1 mut dyn Debug>)
    let mut x = MyData(RefCell::new(&mut ss)); // x: MyData<'1> (不变, '1 无法缩短)

    let y = f(&x, &x); // f(&'2 MyData<'1>, &'2 MyData<'1>) -> MyData<'2> ('2 来自 &'2 x)

    g(y, &x); // g(MyData<'2>, &'3 MyData<'1>) -> MyData<'3> (注意：这直接将 y 的类型代入)
    // 显然 y 的类型上的生命周期与 g 的签名上的不一致：y 与 x 在类型上具有相同的生命周期。
    // 而 x 的生命周期 '1 无法缩短，从而试着把 '2 = '1 代入，得到
    // g(MyData<'1>, &'3 MyData<'1>) -> MyData<'3> 符合 g 的签名。
    // 倒推 f(&'1 MyData<'1>, &'1 MyData<'1>) -> MyData<'1>，嗯，看见 &'1 MyData<'1> (即 &'1 x) 了吗，
    // &'1 x 是一个永久借用！

    drop(x);// error: cannot move out of `x` because it is borrowed
}

struct MyData<'a>(RefCell<&'a mut dyn std::fmt::Debug>);

fn static_data<'any>() -> MyData<'any> {
    MyData(RefCell::new(Box::leak(Box::new(""))))
}
fn f<'a, 'b>(_: &'b MyData<'a>, _: &'b MyData<'a>) -> MyData<'b> {
    static_data()
}
fn g<'a, 'b>(_: MyData<'a>, _: &'b MyData<'a>) -> MyData<'b> {
    static_data()
}
```

而且编译器正确指明了那个永久借用发生的位置！

```rust
error[E0505]: cannot move out of `x` because it is borrowed
  --> src/main.rs:20:10
   |
9  |     let mut x = MyData(RefCell::new(&mut ss));
   |         ----- binding `x` declared here
10 |
11 |     let y = f(&x, &x);
   |               -- borrow of `x` occurs here
...
20 |     drop(x);
   |          ^
   |          |
   |          move out of `x` occurs here
   |          borrow later used here
```

<details>
  <summary>【点击展开】关于原帖，以及我猜来自原帖的读者会有的一些疑问</summary>

对原型的标注 [在这](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=ba2376e9eaef0e5b2dd7305d6b259e19)（我依然简化了一些非常无关问题的代码）。

疑问1：不显式调用 `drop(x)` 不就可以通过代码，需要那么麻烦去弄清楚吗？

回答：这正是我在 [与 drop check 交互](#与-drop-check-交互) 写的，你需要知道这样的代码不是真正有用的。
如果你阅读了全文，当你按照同样方式简单封装一下代码 
([playground][mydata])，就会充分理解编译器指出的问题 —— 两个 `&'1 x` 都被捕捉到了。

[mydata]: https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=aae44ae01b5ab0940ad6688ad37513d1

```rust,no_run
fn fail(x: MyData<'_>) {
    let y = &x + &x;
    let _ = y + &x;
}

error[E0597]: `x` does not live long enough
  --> src/main.rs:71:13
   |
70 | fn fail(x: MyData<'_>) {
   |         -
   |         |
   |         binding `x` declared here
   |         has type `MyData<'1>`
71 |     let y = &x + &x;
   |             ^^-----
   |             |
   |             borrowed value does not live long enough
   |             assignment requires that `x` is borrowed for `'1`
72 |     let _ = y + &x;
73 | }
   |  - `x` dropped here while still borrowed

error[E0597]: `x` does not live long enough
  --> src/main.rs:71:18
   |
70 | fn fail(x: MyData<'_>) {
   |         -
   |         |
   |         binding `x` declared here
   |         has type `MyData<'1>`
71 |     let y = &x + &x;
   |             -----^^
   |             |    |
   |             |    borrowed value does not live long enough
   |             assignment requires that `x` is borrowed for `'1`
72 |     let _ = y + &x;
73 | }
   |  - `x` dropped here while still borrowed
```

疑问2：如何真正解决问题？

原帖当然在滥用生命周期、滥用运算符、滥用内存泄露、滥用内部可变性，不应该那样过度设计程序。

此外，`&'a Invariant<'a>` 是我们需要极力避免的，对于简化后的代码，把 f 和 g 
函数单独看签名似乎都没有过度约束，结合起来形成了过度约束。所以型变中，对于 invariance
是最需要注意的。重新回到出错的地方，我们会注意到有一个 `'3`，它在 `&'3 MyData<'1>` 中似乎有改进的空间

```rust,no_run
let y = f(&x, &x); // f(&'2 MyData<'1>, &'2 MyData<'1>) -> MyData<'2> ('2 来自 &'2 x)
g(y, &x); // g(MyData<'2>, &'3 MyData<'1>) -> MyData<'3> (注意：这直接将 y 的类型代入)
```

当 `'3 = '2` 时，这两行的所有 `&x` 被变成了 `&'2 x`，这是合理的，因为共享借用可以共享同一个生命周期，从而有
([playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=c381fa2d1e927e20b0b42868329c9ecf))

```rust,no_run
let y = f(&x, &x); // f(&'2 MyData<'1>, &'2 MyData<'1>) -> MyData<'2> ('2 来自 &'2 x)
g(y, &x); // g(MyData<'2>, &'2 MyData<'1>) -> MyData<'2> 成立

// 相应的 g 的签名应改为
fn g<'a, 'b>(_: MyData<'b>, _: &'b MyData<'a>) -> MyData<'b> { ... }
```

这就是原帖中，yuyidegit 给的 [`impl<'a, 'b> Add<&'b MyData<'a>> for MyData<'b>`][yuyidegit] 能够通过编译的原因。

[yuyidegit]: https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=04d52ee1137e1e3256b318172d239862

</details>



