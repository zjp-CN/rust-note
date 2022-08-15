# Subtyping and Variance

## 子类型

`T: U` 的含义：
- `T` 是 `U` 的子类型 (subtype)，或者说 `U` 是 `T` 的超类型 (supertype)
- 任何需要使用 `U` 的地方，都能够使用 `T`[^subtyping-wiki]

[^subtyping-wiki]: 可参考 [Wiki](https://en.wikipedia.org/wiki/Subtyping) 的英文描述：
> If `T` is a subtype of `U`, the subtyping relation is often written `T <: U`,
> to mean that any term of type `T` can be safely used in a context where a term of type `U` is expected.

具体而言，对于生命周期这种泛型，`'big: 'small` 的含义：
- `'big` 是 `'small` 的子类型，任何需要使用 `'small` 的地方，都能够使用 `'big`
- `'big` 可当做 `'small` 去使用，因为 `'big` 所描述的代码区域 (region of code) 比 `'small` 更多
- `'big` 包含 `'small`，或者说 `'big` 的生命周期至少和 `'small` 一样长[^big-small]
- 换言之，我们可以忘记某物要存活 `'big` 那么长，只需要记住某物需要存活 `'small` 那么长

[^big-small]: 英文表述为 `'big` outlives/lives at least as long as `'small`

## 型变

Nomicon 一书这样描述和总结了 Rust 中的型变：

* 是一组子类型应该如何组织起来的规则
* 定义了禁用子类型的情况
* 是类型构造器相对于其参数所具有的属性
* 类型构造器 `F` 的型变，就是指其输入的子类型如何影响其输出的子类型
* 在实际场景中，型变 (variance) 这一术语（很多情况下）指“协变” (covariance)
* 几乎所有对型变的考虑都是关于某事应该是协变的还是不变的
* 在 Rust 中见证逆变是相当困难的，尽管它实际上的确存在

类型构造器 (type constructor)：

- 是任何不限制其参数 (unbound arguments) 的泛型类型
- 例如：
  - `Vec` 是一个类型构造器，它接受类型 `T` 并返回 `Vec<T>`
  - `&` 和 `&mut` 是接受两个输入的类型构造器：生命周期和被指向的类型
- 为了方便起见，通常将 `F<T>` 称为类型构造器，这样我们就可以轻松地谈论 `T`

| type constructor | variance                                      | the subtyping of its outputs[^given] | memo                             |
|:----------------:|-----------------------------------------------|--------------------------------------|----------------------------------|
|      `F<T>`      | covariant                                     | `F<Sub>: F<Super>`                   | subtyping "passes through"       |
|      `F<T>`      | contravariant                                 | `F<Super>: F<Sub>`                   | subtyping is "inverted"          |
|      `F<T>`      | invariant                                     | neither the two above                | no subtyping relationship exists |
|     `F<T, U>`    | covariant over `T` and covariant over `U`     | `F<SubT, SubU>: F<SuperT, SuperU>`   | `F` with a single field          |
|     `F<T, U>`    | covariant over `T` and contravariant over `U` | `F<SubT, SuperU>: F<SuperT, SubU>`   | `F` with a single field          |
|     `F<T, U>`    | covariant over `T` and invariant over `U`     | `F<SubT, _>: F<SuperT, _>`           | `F` with a single field          |
|    `F<A> {..}`   | covariant over `A`                            | all uses of A are covariant          | `F` with one or more fields      |
|    `F<A> {..}`   | contravariant over `A`                        | all uses of A are contravariant      | `F` with one or more fields      |
|    `F<A> {..}`   | invariant over `A`                            | neither the two above                | `F` with one or more fields      |

[^given]: given `Sub: Super`

# 参考资料

1. [Reference: subtyping](https://doc.rust-lang.org/reference/subtyping.html)
2. [Nomicon: subtyping](https://doc.rust-lang.org/nomicon/subtyping.html)
3. [Crust of Rust: Subtyping and Variance](https://www.youtube.com/watch?v=iVYWDIW71jk)
4. [Variance in Rust: An intuitive explanation](https://ehsanmkermani.com/2019/03/16/variance-in-rust-an-intuitive-explanation/)
5. [video: Felix Klock - Subtyping in Rust and Clarke's Third Law](https://www.youtube.com/watch?v=fI4RG_uq-WU) with 
   [slides](http://pnkfx.org/presentations/rustfest-berlin-2016/slides.html)
