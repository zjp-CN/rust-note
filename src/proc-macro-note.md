
## 学习经验

过程宏是 Rust 中进阶的内容，它是声明宏的拓展。但它的学习资料比较少，我整理了一些：你可以在
[此链接](https://www.yuque.com/zhoujiping/programming/rust-materials) 网页中搜索【过程宏】找到。

如果你想编写过程宏，那么以下内容是必须掌握的：
1. Rust 几乎所有的语法：因为 Rust 的宏就是在操作 AST
    （或者说 CST？），所以掌握源代码的语法结构是第一步。当你能自如地查阅 
    [Reference](https://doc.rust-lang.org/nightly/reference) 一书，那么这一步就成功了。
2. Rust 的声明宏：声明宏可以单独学习，它完全不涉及过程宏；但过程宏涉及声明宏，而且它们之间有许多相似的地方。
   你可以不需要掌握声明宏高阶模式部分，但你至少要掌握声明宏最通用的部分。
   当你对 [The Little Book of Rust Macros](https://zjp-cn.github.io/tlborm/)
   一书中的大部分内容熟悉时，你可以准备迎接过程宏了。


对我来说，学习过程宏的过程：
1. 从 [Rust Book: ch19-06-macros](https://doc.rust-lang.org/book/ch19-06-macros.html) 中知道过程宏的分类。
2. 从 [Reference: procedural-macros](https://doc.rust-lang.org/nightly/reference/procedural-macros.html)
    中知道过程宏真正的编写框架。
3. 从 [syn/examples](https://github.com/dtolnay/syn/tree/master/examples) 
    中学习如何在特定任务下真实地编写过程宏。你对这四个例子理解地越仔细，那么你就能越快地上手过程宏。
4. 最重要的资料是文档：[quote](https://docs.rs/quote/latest/quote/) 和 [syn](https://docs.rs/syn/latest/syn/)
    。过程宏不像声明宏那样开箱即用，你需要引入别的库，所以你需要掌握这两个库。

谨记 dtolnay 在 [proc-macro-workshop](https://github.com/dtolnay/proc-macro-workshop) 教程中的这些话：

> There is only one profound insight about Rust macro development, and this
> test case begins to touch on it: what makes someone an "expert at macros"
> mostly has nothing to do with how good they are "at macros".
>
> 95% of what enables people to write powerful and user-friendly macro
> libraries is in their mastery of everything else about Rust outside of
> macros, and their creativity to put together ordinary language features in
> interesting ways that may not occur in handwritten code.
>
> You may occasionally come across procedural macros that you feel are really
> advanced or magical. If you ever feel this way, I encourage you to take a
> closer look and you'll discover that as far as the macro implementation
> itself is concerned, none of those libraries are doing anything remotely
> interesting. They always just parse some input in a boring way, crawl some
> syntax trees in a boring way to find out about the input, and paste together
> some output code in a boring way exactly like what you've been doing so far.
> In fact once you've made it this far in the workshop, it's okay to assume you
> basically know everything there is to know about the mechanics of writing
> procedural macros.
>
> To the extent that there are any tricks to macro development, all of them
> revolve around *what* code the macros emit, not *how* the macros emit the
> code. This realization can be surprising to people who entered into macro
> development with a vague notion of procedural macros as a "compiler plugin"
> which they imagine must imply all sorts of complicated APIs for *how* to
> integrate with the rest of the compiler. That's not how it works. The only
> thing macros do is emit code that could have been written by hand. If you
> couldn't have come up with some piece of tricky code from one of those
> magical macros, learning more "about macros" won't change that; but learning
> more about every other part of Rust will. Inversely, once you come up with
> what code you want to generate, writing the macro to generate it is generally
> the easy part.
>
> src: [bitfield/tests/04-multiple-of-8bits.rs](https://github.com/dtolnay/proc-macro-workshop/blob/master/bitfield/tests/04-multiple-of-8bits.rs)


