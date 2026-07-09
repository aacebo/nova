use zyn::proc_macro2::{Ident, Span, TokenStream, TokenTree};
use zyn::syn::Type;

pub fn split_as(input: TokenStream) -> zyn::syn::Result<(TokenStream, Option<Type>)> {
    let tokens: Vec<TokenTree> = input.into_iter().collect();
    let as_pos = tokens.iter().position(|tt| matches!(tt, TokenTree::Ident(id) if *id == "as"));

    match as_pos {
        Some(pos) => {
            let head: TokenStream = tokens[..pos].iter().cloned().collect();
            let tail: TokenStream = tokens[pos + 1..].iter().cloned().collect();
            let ty = zyn::syn::parse2::<Type>(tail)?;
            Ok((head, Some(ty)))
        }
        None => Ok((tokens.into_iter().collect(), None)),
    }
}

pub fn variant_accessor(ty: &Type) -> zyn::syn::Result<Ident> {
    let segment = match ty {
        Type::Path(path) => path.path.segments.last().map(|s| s.ident.to_string()),
        _ => None,
    };

    let accessor = match segment.as_deref() {
        Some("Value") => "as_value",
        Some("Function") => "as_func",
        Some("Routine") => "as_routine",
        _ => {
            return Err(zyn::syn::Error::new_spanned(
                ty,
                "expected an Object variant: Value, Function, or Routine",
            ));
        }
    };

    Ok(Ident::new(accessor, Span::call_site()))
}
