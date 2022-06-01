use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Ident, LitInt, Result,
};

#[proc_macro]
pub fn all_tuples(input: TokenStream) -> TokenStream {
    let Input { start, end, ident } = parse_macro_input!(input);
    let items = (start..end + 1).map(|n| {
                                    let ids = (start..n + 1).map(|n| format_ident!("{ident}{n}"));
                                    quote!(impl_component! {#(#ids),*})
                                });
    let macro_impl_component = quote!(
        macro_rules! impl_component {
            ($($P:ident),*) => {
                impl<F, $($P,)* Content> $crate::Component<( $($P,)* ), Content> for F
                    where F: Fn(&mut $crate::Ui, $($P,)* Content),
                          $( $P: ::std::cmp::PartialEq + ::std::clone::Clone + 'static, )*
                          Content: ::std::ops::FnOnce(&mut Ui)
                {

                    fn call(&self, ui: &mut $crate::Ui, params: ( $($P,)* ), content: Content) {
                        #[allow(non_snake_case)]
                        let ($($P,)*) = params;
                        self(ui, $($P,)* content)
                    }
                }
            };
        }
    );
    quote!(#macro_impl_component #(#items)*).into()
}

struct Input {
    start: u8,
    end:   u8,
    ident: Ident,
}

impl Parse for Input {
    #[rustfmt::skip]
    fn parse(input: ParseStream) -> Result<Self> {
        use syn::token::Comma;
        let start    = input.parse::<LitInt>()?.base10_parse()?;
        let _: Comma = input.parse()?;
        let end      = input.parse::<LitInt>()?.base10_parse()?;
        let _: Comma = input.parse()?;
        let ident    = input.parse()?;
        Ok(Input { start, end, ident })
    }
}
