# Subtyping and Variance


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

[^big-small]: `'big` contains/outlives/lives at least as long as `'small`

variance:
- is a set of rules governing how subtyping should compose
- defines situations where subtyping should be disabled
- is a property that type constructors have with respect to their arguments
- A type constructor `F`'s variance is how the subtyping of its inputs affects the subtyping of its outputs
- covariance is, in practical terms, "the" variance
- Almost all consideration of variance is in terms of whether something should be covariant or invariant
- witnessing contravariance is quite difficult in Rust, though it does in fact exist

A type constructor:
- is any generic type with unbound arguments
- For instance
    - `Vec` is a type constructor that takes a type `T` and returns `Vec<T>`
    - `&` and `&mut` are type constructors that take two inputs: a lifetime, and a type to point to
- For convenience often refer to `F<T>` as a type constructor just so that we can easily talk about `T`


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
