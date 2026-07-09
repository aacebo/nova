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
            Self::SplatKargs(expr) => zyn! {
                __args = ({{ expr }}).args().to_vec();
                __kargs = ({{ expr }}).kargs().clone();
            },
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

pub struct Args {
    pub args: Vec<Arg>,
}

impl Args {
    pub fn tokens(&self) -> TokenStream {
        let positional: Vec<_> = self
            .args
            .iter()
            .filter_map(|arg| match arg {
                Arg::Positional(expr) => Some(zyn! { __args.push(::nova::Value::from({{ expr }})); }),
                Arg::SplatArgs(expr) => Some(zyn! { __args.extend(({{ expr }}).iter().cloned()); }),
                _ => None,
            })
            .collect();

        let named: Vec<_> = self
            .args
            .iter()
            .filter_map(|arg| match arg {
                Arg::Named(key, expr) => {
                    let key = LitStr::new(&key.to_string(), key.span());
                    Some(zyn! { __kwargs.push(({{ key }}, ::nova::Value::from({{ expr }}))); })
                }
                Arg::SplatKargs(expr) => Some(zyn! {
                    for (__k, __v) in ({{ expr }}).iter() {
                        __kwargs.push((__k.as_str(), __v.clone()));
                    }
                }),
                _ => None,
            })
            .collect();

        let has_named = self
            .args
            .iter()
            .any(|arg| matches!(arg, Arg::Named(..) | Arg::SplatKargs(..)));

        let push_kwargs = if has_named {
            zyn! {
                __args.push(::nova::Value::from(::nova::Kwargs::from_iter(__kwargs)));
            }
        } else {
            TokenStream::new()
        };

        zyn! {
            ::nova::Args::try_from(&{
                let mut __args: ::std::vec::Vec<::nova::Value> = ::std::vec::Vec::new();
                let mut __kwargs: ::std::vec::Vec<(&str, ::nova::Value)> = ::std::vec::Vec::new();
                @for (stmt in positional.iter()) { {{ stmt }} }
                @for (stmt in named.iter()) { {{ stmt }} }
                {{ push_kwargs }}
                __args
            }[..]).expect("args! builds a well-formed argument slice")
        }
    }
}

impl Parse for Args {
    fn parse(input: ParseStream) -> zyn::syn::Result<Self> {
        let mut args = Vec::new();
        let mut seen_named = false;

        while !input.is_empty() {
            let arg = input.parse::<Arg>()?;

            match &arg {
                Arg::Named(..) | Arg::SplatKargs(..) => seen_named = true,
                Arg::Positional(..) | Arg::SplatArgs(..) if seen_named => {
                    return Err(input.error("positional argument cannot follow a named argument"));
                }
                _ => {}
            }

            args.push(arg);

            if input.parse::<Option<Token![,]>>()?.is_none() {
                break;
            }
        }

        Ok(Self { args })
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
