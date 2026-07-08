use zyn::proc_macro2::TokenStream;
use zyn::syn::parse::{Parse, ParseStream};
use zyn::syn::{Expr, Ident, LitStr, Token, Type};
use zyn::zyn;

pub enum Arg {
    Positional(Expr),
    Named(Ident, Expr),
    SplatArgs(Expr),
    SplatKargs(Expr),
}

impl Arg {
    pub fn stmt(&self) -> TokenStream {
        match self {
            Self::Positional(expr) => zyn! { __args.push(::nova::Value::from({{ expr }})); },
            Self::Named(key, expr) => {
                let key = LitStr::new(&key.to_string(), key.span());
                zyn! { __kargs.set({{ key }}, {{ expr }}); }
            }
            Self::SplatArgs(expr) => zyn! { __args = ({{ expr }}).to_vec(); },
            Self::SplatKargs(expr) => zyn! { __kargs = ({{ expr }}).clone(); },
        }
    }
}

impl Parse for Arg {
    fn parse(input: ParseStream) -> zyn::syn::Result<Self> {
        if input.peek(Token![*]) && input.peek2(Token![*]) {
            input.parse::<Token![*]>()?;
            input.parse::<Token![*]>()?;
            return Ok(Self::SplatKargs(input.parse::<Expr>()?));
        }

        if input.peek(Token![*]) {
            input.parse::<Token![*]>()?;
            return Ok(Self::SplatArgs(input.parse::<Expr>()?));
        }

        if input.peek(Ident) && input.peek2(Token![=]) && !input.peek2(Token![==]) {
            let key = input.parse::<Ident>()?;
            input.parse::<Token![=]>()?;
            return Ok(Self::Named(key, input.parse::<Expr>()?));
        }

        Ok(Self::Positional(input.parse::<Expr>()?))
    }
}

pub struct Call {
    pub name: Expr,
    pub args: Vec<Arg>,
    pub coerce: Option<Type>,
}

impl Parse for Call {
    fn parse(input: ParseStream) -> zyn::syn::Result<Self> {
        let full = input.parse::<TokenStream>()?;
        let (head, coerce) = crate::coerce::split_as(full)?;
        let head = zyn::syn::parse2::<CallHead>(head)?;

        Ok(Self {
            name: head.name,
            args: head.args,
            coerce,
        })
    }
}

struct CallHead {
    name: Expr,
    args: Vec<Arg>,
}

impl Parse for CallHead {
    fn parse(input: ParseStream) -> zyn::syn::Result<Self> {
        let name = input.parse::<Expr>()?;
        let mut args = Vec::new();
        let mut seen_named = false;

        while input.parse::<Option<Token![,]>>()?.is_some() {
            if input.is_empty() {
                break;
            }

            let arg = input.parse::<Arg>()?;

            match &arg {
                Arg::Named(..) => seen_named = true,
                Arg::Positional(..) if seen_named => {
                    return Err(input.error("positional argument cannot follow a named argument"));
                }
                _ => {}
            }

            args.push(arg);
        }

        Ok(Self { name, args })
    }
}
