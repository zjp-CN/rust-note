# proc_macro2

[`proc_macro2`] 是对 [`proc_macro`] 的包装，因为后者只能用在 proc-macro crate 中。

[`proc_macro`]: https://doc.rust-lang.org/proc_macro/
[`proc_macro2`]: https://docs.rs/proc-macro2/*/proc_macro2

这两个的库的大多类型都相互实现了 `From`，所以可以**相互转化**。但出于习惯[^usually]，我们常常使用 `from` 方法，而不使用
`into` 方法（当然，你要是想使用 `into` 方法，也没有人会阻止你）。

[^usually]: 仅限于过程宏范畴，因为这一说法来自于我对
[dtolnay/syn#35b498](https://github.com/dtolnay/syn/commit/35b498ec501b345a57aa0144a9b22d5fa85d7415) 的观察。

我们可以在内部使用 `proc_macro2` 的类型，而在最后的过程宏函数中把 
`proc_macro2::TokenStream` 转化成 `proc_macro::TokenStream`。

```rust,ignore
#// src: https://github.com/zjp-CN/proc-macro-workshop/blob/master/bitfield/impl/src/lib.rs
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn bitfield(_: TokenStream, input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::Item);
    TokenStream::from(bit::expand(input))
}

#[proc_macro]
pub fn gen(_: TokenStream) -> TokenStream {
    TokenStream::from(gen::generate()) 
}

#[proc_macro_derive(BitfieldSpecifier)]
pub fn derive_bitfield_specifier(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::ItemEnum);
    TokenStream::from(spe::derive_bitfield_specifier_for_enum(input))
}
```

接下来讨论 `TokenStream` 和 `TokenTree`。

## `TokenStream`

[`TokenStream`] 的 [inherent] 方法很少，只有 `fn new() -> Self` 和
`fn is_empty(&self) -> bool`。更重要的是其 trait 实现：

- `IntoIterator`：从 `TokenStream` 到 [`TokenTree`] 只需要 `.into_iter()`
- `FromIterator<TokenStream>`：把多个 `TokenStream` 汇聚成一个 `TokenStream`
- `FromIterator<TokenTree>`：把多个 `TokenTree` 汇聚成一个 `TokenStream`
- `Extend<TokenStream>`：把多个 `TokenStream` 添加进来
- `Extend<TokenTree>`：把多个 `TokenTree` 添加进来

构造 `TokenStream` 并不难，最常用 [`quote::quote!`](./quote.html#quote-与-totokens)。

借助 `Iterator` 相关的泛型实现，不难理解下面的代码 —— 把多个 `TokenStream` 汇总成一个 `TokenStream`：

```rust
use proc_macro2::TokenStream;
fn main() {
    {
        // `Extend<TokenStream>`
        let mut ts = TokenStream::new();
        ts.extend(iter());
        assert_eq!(count(ts), N);
    }
    {
        // `FromIterator<TokenStream>` + impl<I: Iterator> IntoIterator for I
        let ts = TokenStream::from_iter(iter());
        assert_eq!(count(ts), N);
    }
    {
        // `FromIterator<TokenStream>` + `Iterator::collect()`
        let ts: TokenStream = iter().collect();
        assert_eq!(count(ts), N);
    }
    {
        // quote! 的反复插值
        let iter = 0..N;
        assert_eq!(count(quote::quote! {#(#iter)*}), N);
    }
}

#const N: usize = 10;
#
#fn f(i: usize) -> TokenStream {
#    quote::quote! { const _: usize = #i; }
#}
#
#fn iter() -> impl Iterator<Item = TokenStream> { (0usize..N).map(f) }
#
#fn count(ts: TokenStream) -> usize { ts.into_iter().count() }
```

[`TokenStream`]: https://docs.rs/proc-macro2/*/proc_macro2/struct.TokenStream.html
[`TokenTree`]: https://docs.rs/proc-macro2/*/proc_macro2/enum.TokenTree.html
[inherent]: https://doc.rust-lang.org/nightly/reference/glossary.html#inherent-method

## `TokenTree`

[`TokenTree`] 是一个枚举体，它描述一个标记的类别：

```rust,ignore
pub enum TokenTree {
    Group(Group),
    Ident(Ident),
    Punct(Punct),
    Literal(Literal),
}
```

| 类型        | 含义                                                                          |
|-------------|-------------------------------------------------------------------------------|
| [`Group`]   | 由 `{}`、 `()`、 `[]` 等[分隔][`Delimiter`]出来的标记（包括两侧的括号分隔符） |
| [`Ident`]   | 一个标识符，比如 `ABC`、 `_`、 `let`                                          |
| [`Punct`]   | 一个标点，比如 `+`、 `,`、 `$`                                                |
| [`Literal`] | 一个字面值，比如字符 `'a'`、字符串 `"hello"`、数字 `2.3f64`                   |

你很少自己通过 `TokenTree` 解析标记，因为 [`syn`] 提供大量基础而通用的节点类型，它们都实现了 [`Parse`] trait。

一个很好的必须使用 `TokenTree` 的案例是 proc-macro-workshop 的
`Seq`，你可以参考我的[解答](https://github.com/zjp-CN/proc-macro-workshop/blob/master/seq/src/lib.rs)。

[`syn`]: https://docs.rs/syn
[`Parse`]: ./syn.html#parse-trait

[`Group`]: https://docs.rs/proc-macro2/*/proc_macro2/struct.Group.html
[`Ident`]: https://docs.rs/proc-macro2/*/proc_macro2/struct.Ident.html
[`Punct`]: https://docs.rs/proc-macro2/*/proc_macro2/struct.Punct.html
[`Literal`]: https://docs.rs/proc-macro2/*/proc_macro2/struct.Literal.html
[`Delimiter`]: https://docs.rs/proc-macro2/*/proc_macro2/enum.Delimiter.html
