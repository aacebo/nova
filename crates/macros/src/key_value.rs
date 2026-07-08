use zyn::syn::parse::{Parse, ParseStream};
use zyn::syn::{Expr, Token};

pub struct KeyValue {
    pub key: Expr,
    pub value: Expr,
}

impl Parse for KeyValue {
    fn parse(input: ParseStream) -> zyn::syn::Result<Self> {
        let key = input.parse::<Expr>()?;
        input.parse::<Token![,]>()?;
        let value = input.parse::<Expr>()?;
        Ok(Self { key, value })
    }
}
