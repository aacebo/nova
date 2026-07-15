mod args;
mod coerce;
mod diagnostics;
mod key_value;

use zyn::proc_macro2::TokenStream;
use zyn::syn::{Expr, parse_macro_input};
use zyn::zyn;

use crate::args::{Args, Call};
use crate::diagnostics::{Diagnostic, SeverityDiagnostic};
use crate::key_value::KeyValue;

#[proc_macro]
pub fn diagnostic(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let SeverityDiagnostic { severity, diagnostic } = parse_macro_input!(input as SeverityDiagnostic);
    diagnostic.tokens(zyn! { {{ severity }} }).into()
}

#[proc_macro]
pub fn info(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let diagnostic = parse_macro_input!(input as Diagnostic);
    diagnostic.tokens(zyn! { ::nova::Severity::Info }).into()
}

#[proc_macro]
pub fn warn(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let diagnostic = parse_macro_input!(input as Diagnostic);
    diagnostic.tokens(zyn! { ::nova::Severity::Warn }).into()
}

#[proc_macro]
pub fn error(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let diagnostic = parse_macro_input!(input as Diagnostic);
    diagnostic.tokens(zyn! { ::nova::Severity::Error }).into()
}

#[proc_macro]
pub fn get(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    lookup(input, zyn! { get })
}

#[proc_macro]
pub fn get_mut(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    lookup(input, zyn! { get_mut })
}

#[proc_macro]
pub fn set(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let KeyValue { key, value } = parse_macro_input!(input as KeyValue);
    zyn! { scope.set({{ key }}, {{ value }}) }.into()
}

#[proc_macro]
pub fn has(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let key = parse_macro_input!(input as Expr);
    zyn! { scope.has({{ key }}) }.into()
}

#[proc_macro]
pub fn del(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let key = parse_macro_input!(input as Expr);
    zyn! { scope.del({{ key }}) }.into()
}

#[proc_macro]
pub fn args(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let args = parse_macro_input!(input as Args);
    args.tokens().into()
}

#[proc_macro]
pub fn call(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let Call { name, args, coerce } = parse_macro_input!(input as Call);
    let stmts: Vec<_> = args.iter().map(args::Arg::stmt).collect();
    let invoke = zyn! {
        {
            let mut __args: ::std::vec::Vec<::nova::reflect::Value> = ::std::vec::Vec::new();
            let mut __kargs = ::nova::template::KArgs::new();
            @for (stmt in stmts.iter()) {
                {{ stmt }}
            }
            scope.call({{ name }}, ::nova::template::Args::new(__args, __kargs))?
        }
    };

    let expanded = match coerce {
        Some(ty) => zyn! {
            <{{ ty }} as ::std::convert::TryFrom<::nova::template::Pointer>>::try_from({{ invoke }})?
        },
        None => invoke,
    };

    expanded.into()
}

fn lookup(input: proc_macro::TokenStream, method: TokenStream) -> proc_macro::TokenStream {
    let (key, ty) = match coerce::split_as(input.into()) {
        Ok(parts) => parts,
        Err(err) => return err.to_compile_error().into(),
    };

    let key = match zyn::syn::parse2::<Expr>(key) {
        Ok(key) => key,
        Err(err) => return err.to_compile_error().into(),
    };

    match ty {
        None => zyn! { scope.{{ method }}({{ key }}) }.into(),
        Some(ty) => match coerce::variant_accessor(&ty) {
            Ok(accessor) => zyn! { scope.{{ method }}({{ key }}).filter(|__slot| __slot.{{ accessor }}().is_some()) }.into(),
            Err(err) => err.to_compile_error().into(),
        },
    }
}
