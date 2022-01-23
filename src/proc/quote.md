# `quote`

这里主要结合编写经验来总结 quote 的使用方式，大部分内容是对 quote 文档的翻译和重新组织。

如果内容上有出入，请以 quote 文档为准。

## 基础知识

Rust 的过程宏所做的事情就是：
1. 从<abbr title="input">输入</abbr>中获取<abbr title="a stream of tokens">标记流</abbr>；
2. 处理甚至生成标记流；
3. 把处理过的或者新的标记流传回编译器；
4. 编译器将这些标记流编译进调用者的 crate。

[`quote`](https://docs.rs/quote) crate 把 Rust 语法树的数据结构转化为源代码的标记 
(tokens)，在上述过程中，`quote` 在生成标记并返回给编辑器的环节提供了一种解决方案。

`quote` 提出一种概念 quasi-quoting[^quasi-quoting]
，把我们所编写的代码视为数据。从而让我们写出看起来像是文本编辑器或者 
IDE 里的那种代码：写这种代码时让我们享受到 IDE 提供的大括号匹配、语法突出显示、缩进以及自动补全等功能。

但是这种代码不直接编译到当前 crate 
中，编写它们就像处理数据一样，我们可以传递、改变这些数据，最后把它们当作标记传回编译器，编译进调用者的 crate。

[^quasi-quoting]: 直白翻译为“类引述”、“类引用”。我的理解为，“写起来像原来的代码一样”。quoting 取 
"repeat or copy out (words from a text or speech written or spoken by another person)" 本义 —— 重复、复制。

一个例子是 [`serde`](https://serde.rs/) crate 提供的过程宏，它使用类似于下面的代码：`quote!`
内的代码看起来像我们在 IDE 中所写的那种代码，但使用 `#var`
进行插值（把运行时的变量插入到相应位置上），这在形式上如同声明宏的 `$var` 插值。

```rust,ignore
let tokens = quote! {
    struct SerializeWith #generics #where_clause {
        value: &'a #field_ty,
        phantom: core::marker::PhantomData<#item_ty>,
    }

    impl #generics serde::Serialize for SerializeWith #generics #where_clause {
       fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            #path(self.value, serializer)
        }
    }

    SerializeWith {
        value: #value,
        phantom: core::marker::PhantomData::<#item_ty>,
    }
}; 
```

此外，`quote` 虽然受过程宏用例驱动开发，但它实际是可实现通用目的的 Rust 库，并不特定于过程性宏。

`quote` 有三个宏和三个 trait。

其中 [`TokenStreamExt`](https://docs.rs/quote/latest/quote/trait.TokenStreamExt.html) 仅仅是对 [`TokenStream`][`TokenStream`]
提供一些封装的、内部使用的方法，它的设计模式可参考
[此处说明](https://zjp-cn.github.io/api-guidelines/future-proofing.html#%E5%B0%81%E8%A3%85%E7%9A%84-traits-%E9%9A%94%E7%BB%9D%E4%B8%8B%E6%B8%B8%E7%9A%84%E5%AE%9E%E7%8E%B0)。

[`TokenStream`]: https://docs.rs/proc-macro2/*/proc_macro2/struct.TokenStream.html

## `format_ident!` 与 `IdentFragment` trait

它们用于拼接标识符。

与 [`format!`](https://doc.rust-lang.org/nightly/alloc/macro.format.html) 类似，
[`format_ident!`](https://docs.rs/quote/*/quote/macro.format_ident.html) 具有以下功能，比如：
1. 除了 `{}` 插值，还支持位置参数和命名参数插值，如 `format_ident!("{0}", arg)` 和 `format_ident!("{arg}", arg)`
2. 支持一些有限的[格式化][std::fmt]的方式：
	* `{}` ⇒ 按照 [`IdentFragment`][`IdentFragment`] 格式化
	* `{:o}` ⇒ [`Octal`](`std::fmt::Octal`) 八进制格式化
	* `{:x}` ⇒ [`LowerHex`](`std::fmt::LowerHex`) 小写字母格式化
	* `{:X}` ⇒ [`UpperHex`](`std::fmt::UpperHex`) 大写字母格式化
	* `{:b}` ⇒ [`Binary`](`std::fmt::Binary`) 二进制格式化
    ```rust,ignore
    let num: u32 = 10;

    let decimal = format_ident!("Id_{}", num);
    assert_eq!(decimal, "Id_10");

    let octal = format_ident!("Id_{:o}", num);
    assert_eq!(octal, "Id_12");

    let binary = format_ident!("Id_{:b}", num);
    assert_eq!(binary, "Id_1010");

    let lower_hex = format_ident!("Id_{:x}", num);
    assert_eq!(lower_hex, "Id_a");

    let upper_hex = format_ident!("Id_{:X}", num);
    assert_eq!(upper_hex, "Id_A");
    ```

[`IdentFragment`]: https://docs.rs/quote/*/quote/trait.IdentFragment.html
[std::fmt]: https://doc.rust-lang.org/nightly/alloc/fmt/index.html

但它也与 `format!` 不同，因为：
1. `format!` 使用 `Display` 方式格式化 `{}`，而 `format_ident!` 使用 `IdentFragment`，而
    `IdentFragment` trait 只对有限的类型实现，比如无符号整数和字符串
2. [`Ident`][`Ident`] 参数如果存在 `r#` 前缀的话，会把 `r#` 去除掉（由 `IdentFragment` 提供的）
3. `format_ident!` 的结果为 [`Ident`][`Ident`] 类型，而不是 `String` 类型。`Ident` 与任何实现
    `AsRef<str>` 的类型[可以比较相等](https://docs.rs/proc-macro2/*/proc_macro2/struct.Ident.html#impl-PartialEq%3CT%3E)

[`Ident`]: https://docs.rs/proc-macro2/*/proc_macro2/struct.Ident.html


标识符 `Ident` 类型表示 Rust 代码的一个
<abbr title="word">字</abbr>，可能是关键字或合法的变量名。标识符至少由一个
Unicode 代码点组成，其中第一个代码点具有 `XID_Start` 属性，其余代码点具有 `XID_Continue`
属性。注意：
- 空字符串 `""` 不是标识符，应该使用 `Option<Ident>` 表示它。
- 生命周期不是标识符，应该使用 `syn::Lifetime`。

除了使用 `quote::format_ident!`，你还可以直接使用 `proc_macro2` 或者 `syn`[^Ident] 里的 `Ident::new(&str, Span::call_site())`
构造任意 Rust 标识符（包括关键字）。但 `format_ident!` 与后者方式的区别在于，`format_ident!` 
可以构造出<abbr title="raw identifier">原生字符串</abbr> `r#`。

在解析标识符时：
1. [`syn::parse`](https://docs.rs/syn/*/syn/parse/trait.Parse.html) 可以解析 Rust 关键字之外的标识符。
2. 通过 `input.call(syn::Ident::parse_any)` 方式可以达到解析任意标识符（包括关键字）的目的。
3. 通过 `input.call(syn::Ident::unraw)` 方式可以达到去除 `r#` 前缀的目的。

[^input]: [`proc_macro2::Ident`][`Ident`]、
[`proc_macro::Ident`](https://doc.rust-lang.org/proc_macro/struct.Ident.html) 和
[`syn::Ident`](https://docs.rs/syn/*/syn/struct.Ident.html) 都定义了类似的数据结构。

从 `Ident::new` 的签名中，可以看到，构造标识符需要提供第二个参数 [`Span`][`Span`]
，它表示源代码的区域范围，意味着标识符在某个范围内的“<abbr title="hygiene">卫生性</abbr>”。
`format_ident!` 在这方面有如下说明：
- 最后创建的标识符使用第一个 `Ident` 参数的范围
```rust,ignore
// 如果 `ident` 是一个被解析的标识符，那么这段的 `my_ident` 会继承 ident 的范围
let my_ident = format_ident!("My{}{}", ident, "IsCool");
assert_eq!(my_ident, "MyIdentIsCool");
```
- 当无标识符可提供时，默认调用 
  [`Span::call_site`](https://docs.rs/proc-macro2/*/proc_macro2/struct.Span.html#method.call_site)
  表示当前过程宏被调用的范围
```rust,ignore
format_ident!("MyIdent")
```
- 也可以使用 `span = ` 指定范围
```rust,ignore
let my_span = /* ... */;
format_ident!("MyIdent", span = my_span);
```


[`Span`]: https://docs.rs/proc-macro2/*/proc_macro2/struct.Span.html

## `quote!` 与 `ToTokens`

### 基础知识

`quote!` 是整个 `quote` crate 提供的最主要的功能：
1. 该宏进行变量插值：任何实现了 `ToTokens` trait 的类型都能插值
2. 宏的结果为 `proc_macro2::TokenStream` 类型：如果要返回给编译器（即作为过程宏的输出），则使用 `.into()` 或者 
    `proc_macro::TokenStream::from` 把它转化为 `proc_macro::TokenStream` 类型

一个 derive 过程宏的代码框架：

```rust,ignore
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(MyMacro)]
pub fn my_macro(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // Build the output, possibly using quasi-quotation
    let expanded = quote! {
        // ...
    };

    // Hand the output tokens back to the compiler
    TokenStream::from(expanded)
}
```

函数内的代码就是我们编写过程宏的主体，完成了过程宏的第 2 步：处理和生成标记流。具体来说：
1. `syn::parse_macro_input!` 把获取到的标记流转化成实现了 `syn::parse::Parse` 的类型
2. 然后基于 `syn` 丰富的数据结构，处理标记流
3. 利用 `quote::quote!` 把标记流重组成 `proc_macro2::TokenStream`，其间可把实现了 `quote::ToTokens` 的数据插值
4. 最后把 `proc_macro2::TokenStream` 转化成 
    `proc_macro::TokenStream`[^proc_macro2-proc_macro::TokenStream]，它们之间实现了
    [双向转化](https://docs.rs/proc-macro2/*/proc_macro2/struct.TokenStream.html#impl-From%3CTokenStream%3E)

[^proc_macro2-proc_macro::TokenStream]: 这一步很好理解，因为过程宏的函数签名只有
`proc_macro::TokenStream`。此外，一个基本事实是，忽略 [`unicode-xid`](https://docs.rs/unicode-xid) 
依赖，`proc_macro2` 对 `proc_macro` 进行了包装，`quote` 基于和拓展了 `proc_macro2` 的功能，而 
`syn` 基于和拓展了 `quote` 与 `proc_macro2`。

### 插值

`quote!` 使用 `#var` 语法插值，这类似于 `macro_rules!` 的 `$var` 
插值方式。这里说的“插值”，具体指：把当前作用域里的变量 `var` 插入到输出标记的某处。

具体能插入哪些东西呢？或者说如何控制插入的标记的含义呢？

与声明宏使用 13 种分类片段符 
([fragment specifiers](https://zjp-cn.github.io/tlborm/macros/minutiae/fragment-specifiers.html)) 
进行解析和生成标记的做法不同，过程宏使用了不同的方式[^ToTokens]：使用
[`syn::parse`](https://docs.rs/syn/latest/syn/parse/index.html) 解析标记，并使用 [`quote::ToTokens`][`ToTokens`]
trait 生成标记。

[`ToTokens`]: https://docs.rs/quote/latest/quote/trait.ToTokens.html

任何实现了 `ToTokens` 的类型都能通过 `quote!` 插值，而且至少有以下类型：
1. 在 `quote::ToTokens` 实现了的 Rust 大部分的原始类型：各种数值、常用字符串、bool、Option、Box、Rc 等
2. 在 `quote::ToTokens` 实现了的 `proc_macro2` 的主要类型：Group、Ident、Punct、Literal、TokenTree、TokenStream
3. `syn` 内定义的[大部分类型](https://docs.rs/syn/latest/syn/macro.parse_macro_input.html?search=totokens)

[^ToTokens]: 事实上，`quote!` 自身是一个声明宏，而且只以 `tt` 分类符的方式匹配任意多的标记，其背后也大量使用声明宏。

与声明宏的[反复替换](https://zjp-cn.github.io/tlborm/patterns/repetition-replacement.html)功能类似，`quote!`
也支持反复插值，但有以下特点：
- 反复插值其实是在迭代每个元素时，给每个元素插入一份<abbr title="repetition body">重复体</abbr>的代码副本
- `var` 可以是任意满足 `Iterator` 和 `ToTokens` 要求[^ToTokens-Iterator]的类型：比如 `Vec`、`BTreeSet`、实现了
  `Iterator::Item: ToTokens` 的迭代器类型
- 语法（例子）：
    - `#(#var)*`
    - `#(#var),*` ⇒ 每一项之间用 `,` 分隔
    - `#( struct #var; )*`、`#( + #var; )*` ⇒ 可以包含其他标记
    - `#( #k => println!("{}", #v), )*` ⇒ 一次包含多个插值
    ```rust,ignore
    // lib.rs
    use quote::quote;
    use proc_macro::TokenStream;

    #[proc_macro]
    pub fn test(_: TokenStream) -> TokenStream {
        let range = 1..4;
        let q = quote!( 0 #(+ #range)*);
        println!("{}", q); // 0 + 1i32 + 2i32 + 3i32
        dbg!(q).into() // q = TokenStream [ ... ]
    }

    // main.rs
    fn main() {
        let q = path_to::test!();
        dbg!(q); // q = 6
    }
    ```

[^ToTokens-Iterator]: 这里的实现细节有些复杂，感兴趣的话见[源码](https://docs.rs/quote/*/src/quote/runtime.rs.html)。一个特例是
`Vec`，它没有实现 `ToTokens`，但它依然可用于反复插值。

### 卫生性

1. 所有被插值的标记保留了其 `ToTokens` 实现提供的 `Span` 信息
2. `quote!` 生成的非插值标记的 `Span` 为
    [`Span::call_site()`](https://docs.rs/proc-macro2/1.0/proc_macro2/struct.Span.html#method.call_site)
3. 如果需要生成不同于上述 `Span` 的标记，需使用 [`quote_spanned!`][`quote_spanned!`]

[`quote_spanned!`]: https://docs.rs/quote/*/quote/macro.quote_spanned.html

### 细节补充

- `quote` 和 `syn` 是好搭档，你可以在 `syn` 的 [examples](https://github.com/dtolnay/syn/tree/master/examples) 下找到实操的综合例子
- 你可以把最终生成的 `proc_macro2::TokenStream` 拆成多个部分，即利用插值把局部标记组合起来，比如：
    ```rust,ignore
    let type_definition = quote! {...};
    let methods = quote! {...};

    let tokens = quote! {
        #type_definition
        #methods
    };
    ```
- 把构造标识符的步骤放到 `quote!` 之前：
    ```rust,ignore
    // 错误做法：在 `quote!` 内部构造标识符并不会把各部分标识符拼接起来
    // `_` 和 `ident` 代表的标识符不会组合成一个，如果 `ident` 代表 `x`，那么这如同 `_ x`  
    quote! { let mut _#ident = 0; } 

    // 正确做法：在 `quote!` 内只保存 `#var` 语法，就如同引用一样 （quasi-quoting）
    let varname = format_ident!("_{}", ident);
    quote! { let mut #varname = 0; }

    // 正确做法
    let concatenated = format!("_{}", ident);
    let varname = syn::Ident::new(&concatenated, ident.span());
    quote! { let mut #varname = 0; }
    ```
- 调用方法时，尤其是泛型方法，使用 `<Type>::func()` 语法插值，如：
    ```rust,ignore
    // 错误做法：虽然这有时可以生效（比如 `field_type` 是 `String`）
    // 但如果 `field_type` 是 `Vec<i32>`，那么这种方法如同手写的 `Vec<i32>::new()`
    // 而正确写法应为 `Vec::<i32>::new()` 或者 `<Vec<i32>>::new()`
    // 即使为 `String`，这种 `<String>::new()` 语法实际也是正确的
    quote! { let value = #field_type::new(); }

    // 正确做法
    quote! { let value = <#field_type>::new(); }

    // 正确做法
    quote! { let value = <#field_type as core::default::Default>::default(); }
    ```
- 在文档注释中插值：
    ```rust,ignore
    // 错误做法：这不会把 `#ident` 的值插入进来
    quote! {
        /// try to interpolate: #ident
        ///
        /// ...
    }

    // 错误做法：这不会把 `#ident` 的值插入进来
    quote! { #[doc = "try to interpolate: #ident"] }

    // 错误做法：`#[doc]` 属性在 `quote!` 中不支持调用宏（比如这里的 `stringify!` 以及 `include_str!`）
    quote! { #[doc = concat!("try to interpolate: ", stringify!(#ident))] }

    // 正确做法：把需要插入的值（甚至那所涉及的部分）放入变量，使用 `#[doc = #msg]` 语法插值
    let msg = format!(...);
    quote! {
        #[doc = #msg]
        ///
        /// 其他无需插值的注释内容...
    }
    ```
- 使用 [`syn::Index`](https://docs.rs/syn/1.0/syn/struct.Index.html) 对元组或者元组结构体索引插值：
    ```rust,ignore
    // 错误做法
    let i = 0usize..self.fields.len();
    // 以下代码会展开成不正确的语法 `0 + self.0usize.heap_size() + self.1usize.heap_size() + ...` 
    quote! { 0 #( + self.#i.heap_size() )* }

    // 正确做法
    let i = (0..self.fields.len()).map(syn::Index::from);
    // 以下代码展开成正确的语法 `0 + self.0.heap_size() + self.1.heap_size() + ...`
    quote! { 0 #( + self.#i.heap_size() )* }
    ```

## `quote_spanned!`

`quote_spanned!` 与 `quote!` 的唯一区别在于“[卫生性](quote-syn.md#卫生性)”，即 `quote_spanned!` 可以手动指明 `span`：
```rust,ignore
quote_spanned! {span=>
    // 这里的内容与 `quote!` 一致，也有插值功能
};
```

`span=>` 是作者倡导的一种的书写方式，用以表明 span 表达式在过程宏的上下文中求值，而剩余的标记在生成的代码中求值。

关于 `quote!`、`quote_spanned!` 和声明宏的实际案例，见 [`assert_sync!`](./study-case.md#assert_sync)


