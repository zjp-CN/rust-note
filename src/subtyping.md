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
- defines situations where subtyping should be disabled.
- is a property that type constructors have with respect to their arguments.
