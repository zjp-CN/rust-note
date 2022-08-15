# 理解生命周期与型变

## 生命周期的基本观点

生命周期属于 Rust 独有的最基本概念。树立一些基本观点[^fundamental]，能让你理解生命周期更容易。

1. 是 **你写出的代码生成、控制生命周期**，而不是生命周期控制你的代码[^Shepmaster]
  * 生命周期 **不会改变你的代码**
  * 生命周期对你的代码是 **描述性**的，而 *不是规束性的* (lifetimes are **descriptive**, not *prescriptive*)
  * 生命周期关注的是 **当前内存位置将有效多长时间**
2. 生命周期有两种：
  * 引用的生命周期：使用引用的范围（引用包括独占和共享）
  * 值的生命周期：值被释放之前（或者换句话说，在值的析构函数运行之前）的范围
3. NLL 下的生命周期是基于控制流图的，而不是词法范围[^RFC-NLL]
  * NLL 是当前 Rust 借用检查的一部分，虽然 Rust 使用者无需深入了解，但你的借用出现问题，与 NLL 直接相关
    * 如果编译器按照以下“三段式”报告借用、生命周期的错误，那么它们就是 NLL 的产物
      * 首先，产生了值的借用
      * 然后，产生了使引用无效的操作
      * 最后，在引用无效后，下一次该引用被使用了
    * “错误发生的地点”仍然因为第二步的操作
      * 也就是说，错误是由于两次使用引用之间，执行了无效操作导致的 ，而不是在失效操作之后使用引用导致的
      * 这实际上更准确地反映了未定义行为的定义 ：即，执行非法写入是导致未定义行为的原因，但由于后面的使用，写入是非法的
  * 生命周期曾经是词法的：一个引用/值一直延续到块结束，从而其生命周期一直延续到块结束，从而生命周期是连续存在的
  * 生命周期现在是非词法的（即 NLL, non-lexical lifetime）：
    * 引用（从而其生命周期）可以在最后出现的地方终止
    * 值的生命周期可以在最后出现的地方终止，但会受析构函数、 drop 的顺序影响[^desctructor]
    * 确定生命周期的方式是通过求解约束进行的，从词法范围看，生命周期可以存在洞，从而不连续
4. 生命周期还属于泛型范畴
  * 典型场景是使用泛型生命周期参数进行标注
    * 从而告诉 Rust 多个引用（或值）的泛型生命周期参数之间如何相互联系
    * 换句话说，标注生命周期，就是与 Rust 编译器立下双向契约：
      * 如果你的代码符合你所标注的含义，编译通过 
      * 如果你的代码违反你所标注的含义，编译器拒绝并发出错误

[^fundamental]: 注意，这是我理解生命周期的方式，如果有错误，欢迎探讨。

[^Shepmaster]: [@Shepmaster](https://stackoverflow.com/a/32300133/15448980) 所言。

[^RFC-NLL]: 参考 [RFC 2094: NLL](https://rust-lang.github.io/rfcs/2094-nll.html)，或者我对此的[全文翻译](https://zjp-cn.github.io/translation/2094-nll-zh.html)。

[^desctructor]: 参考 [Reference: destructors](https://doc.rust-lang.org/reference/destructors.html)。

## Rust 中的型变

https://users.rust-lang.org/t/anotating-lifetimes-manually/77938/3?u=vague

```rust
// src: https://doc.rust-lang.org/stable/book/ch10-03-lifetime-syntax.html#generic-lifetimes-in-functions
fn main() {
    let x = String::from("hello");
    let mut y = String::from("");

    let result = longest(&x, &y);
    //drop(y);
    //y += " world";
    println!("The longest string is {:?}", result);
}

fn longest<'ret, 'x: 'ret, 'y: 'ret>(x: &'x str, y: &'y str) -> &'ret str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}
```
