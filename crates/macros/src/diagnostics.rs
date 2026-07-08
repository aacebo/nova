use zyn::proc_macro2::TokenStream;
use zyn::syn::parse::{Parse, ParseStream};
use zyn::syn::punctuated::Punctuated;
use zyn::syn::{Expr, LitStr, Token};
use zyn::zyn;

pub struct Diagnostic {
    fmt: LitStr,
    fmt_args: Vec<Expr>,
    children: Vec<Expr>,
}

impl Diagnostic {
    pub fn tokens(self, severity: TokenStream) -> TokenStream {
        let Self { fmt, fmt_args, children } = self;

        let base = zyn! {
            ::nova::Diagnostic::new(::nova::Traced::trace_id(&scope))
                .sev({{ severity }})
                .message(::std::format!({{ fmt }} @for (arg in fmt_args.iter()) { , {{ arg }} }))
        };

        if children.is_empty() {
            base
        } else {
            zyn! {
                {
                    let mut __d = {{ base }};
                    @for (child in children.iter()) {
                        __d = __d.child({{ child }});
                    }
                    __d
                }
            }
        }
    }
}

impl Parse for Diagnostic {
    fn parse(input: ParseStream) -> zyn::syn::Result<Self> {
        let fmt = input.parse::<LitStr>()?;
        let mut fmt_args = Vec::new();

        while input.peek(Token![,]) {
            input.parse::<Token![,]>()?;

            if input.is_empty() || input.peek(Token![;]) {
                break;
            }

            fmt_args.push(input.parse::<Expr>()?);
        }

        let mut children = Vec::new();

        if input.peek(Token![;]) {
            input.parse::<Token![;]>()?;
            let content;
            zyn::syn::bracketed!(content in input);
            children = Punctuated::<Expr, Token![,]>::parse_terminated(&content)?
                .into_iter()
                .collect();
        }

        Ok(Self { fmt, fmt_args, children })
    }
}

pub struct SeverityDiagnostic {
    pub severity: Expr,
    pub diagnostic: Diagnostic,
}

impl Parse for SeverityDiagnostic {
    fn parse(input: ParseStream) -> zyn::syn::Result<Self> {
        let severity = input.parse::<Expr>()?;
        input.parse::<Token![,]>()?;
        let diagnostic = input.parse::<Diagnostic>()?;
        Ok(Self { severity, diagnostic })
    }
}
