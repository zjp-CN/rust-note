[Cargo 1.60 增加了两种 features 的新语法](https://blog.rust-lang.org/2022/04/07/Rust-1.60.0.html#new-syntax-for-cargo-features)：
- 使用 `dep:` 来阻止隐式生成与可选依赖同名的 feature 名
- 使用 `?` 来阻止隐式启用可选依赖及其被指定的 feature

考虑以下项目布局的 features 结构：

```toml
# in root `Cargo.toml` = ./Cargo.toml
[dependencies]
a = { path = "./a", optional = true, default-features = false}
b = { path = "./b", optional = true, features = ["bar"]}

[workspace]
members = ["./a", "./b"]

# in the `Cargo.toml` of crate `a` = ./a/Cargo.toml
[features]
foo = []
baz = []

# in the `Cargo.toml` of crate `b` = ./b/Cargo.toml
[features]
bar = []
```

如果在 root `Cargo.toml` 中定义以下一种 feature （每行独立地看）：

| \[features\] table in root `Cargo.toml`    |                 `a`                |                 `b`                |               `a/foo`              |               `a/baz`              |               `b/bar`              |
|--------------------------------------------|:----------------------------------:|:----------------------------------:|:----------------------------------:|:----------------------------------:|:----------------------------------:|
| `a = ["dep:a"]`                            | <font color="#47f782">**√**</font> |                **f**               |                **f**               |                **f**               |                **f**               |
| `use-a-with-its-name-disabled = ["dep:a"]` | <font color="#2a70fc">**★**</font> |                **f**               | <font color="#f98686">**×**</font> | <font color="#f98686">**×**</font> |                **f**               |
| `maybe-foo = ["a?/foo"]`                   |                **f**               |                **f**               |                **f**               |                **f**               |                **f**               |
| `a-foo = ["a/foo"]`                        | <font color="#47f782">**√**</font> |                **f**               | <font color="#47f782">**√**</font> |                **f**               |                **f**               |
| `bar = ["dep:b", "a?/baz"]`                |                **f**               | <font color="#2a70fc">**★**</font> |                **f**               |                **f**               | <font color="#2a70fc">**★**</font> |
| `bar-and-maybe-baz = ["b", "a?/baz"]`      |                **f**               | <font color="#47f782">**√**</font> |                **f**               |                **f**               | <font color="#47f782">**√**</font> |
| `bar-and-baz = ["b", "a/baz"]`             | <font color="#47f782">**√**</font> | <font color="#47f782">**√**</font> |                **f**               | <font color="#47f782">**√**</font> | <font color="#47f782">**√**</font> |

记号说明：

|               symbol               | whether the column feature <br> is enabled via the row feature | whether the column feature is usable <br> via `--features xx` or `--all-features` flag |
|:----------------------------------:|:--------------------------------------------------------------:|:--------------------------------------------------------------------------------------:|
|                **f**               |                               no                               |                                           yes                                          |
| <font color="#47f782">**√**</font> |                               yes                              |                                           yes                                          |
| <font color="#2a70fc">**★**</font> |                               yes                              |                                           no                                           |
| <font color="#f98686">**×**</font> |                               no                               |                                           no                                           |


<details>
<summary><b><i>也就是说【点击展开】</i></b></summary>

- **f** means the column feature is **not enabled** via the row feature and is **usable** via `--features xx` or `--f` flag
- <font color="#47f782">**√**</font> means the column feature is **implicitly enabled** via the row feature and also **usable**
- <font color="#2a70fc">**★**</font> means the column feature is **implicitly enabled** via the row feature but in **no way to be explicitly enabled** via flags mentioned above
- <font color="#f98686">**×**</font> means the column feature is **no way to be enabled** via the row feature and **no way to be explicitly enabled** via flags mentioned above

</details>

如果你试图把上述表格定义的所有 features 放在一起，你会遇到错误：因为 `dep:` 语法会导致可选依赖不再享有隐式的同名 feature。

具体来说，一旦存在 `dep:a`，那么除非使用 `a = ["dep:a"]` 显示声明 `a` 这个 feature[^dep:a]，否则无法使用 `a` 及 `a` 下面的 features，即
```toml
# in root `Cargo.toml`
[features]
no-a-any-more = ["dep:a"]
this-line-causes-error = ["a"]
this-line-causes-error-too = ["a/foo"]
```

[^dep:a]: 如果定义 `a = ["dep:a"]`，那么依然可以使用 `a` 下面的 features；但如果定义上述 
          `use-a-with-its-name-disabled = ["dep:a"]`，则不再能使用 `a` 下面的 features 了。

<details>
<summary><b><i>错误【点击展开】</i></b></summary>

```text
Package `...` does not have feature `a`. It has an optional dependency with that name,
but that dependency uses the "dep:" syntax in the features table, so it does not have an implicit feature with that name.

或者

feature `this-line-causes-error` includes `a`, but `a` is an optional dependency without an implicit feature.
Use `dep:a` to enable the dependency.
```
</details>

另一个问题出现了，以下定义是合法的吗？

```toml
# in root `Cargo.toml`
[features]
a = ["dep:a"]
foo = ["a/foo"]
another-a = ["dep:a"]
another-a2 = ["a"]
```

<details>
<summary><b><i>答案【点击展开】</i></b></summary>

是合法的。因为 `a = ["dep:a"]` 相当于显示定义 `a` 这个 feature，而 `a/foo` 是合法的 feature 名，所以 `foo = ["a/foo"]` 是合法的。

`another-a = ["dep:a"]` 和 `another-a2 = ["a"]` 都是合法的，而且显然它们代表不同的含义：
- `another-a` 只表明开启该 feature 意味着开启可选依赖 `a` （如果依赖指定了）
- `another-a2` 只表明开启该 feature 意味着开启 `a` feature

但由于这里的 `a` feature 与隐式生成的 `a` feature 功能相同，所以

</details>
