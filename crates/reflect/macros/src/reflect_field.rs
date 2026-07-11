use quote::quote;

use crate::{parse, reflect_meta, reflect_visibility};

pub fn build(field: &syn::Field, index: usize, is_named: bool) -> Option<proc_macro2::TokenStream> {
    let attr = parse::field_attr(&field.attrs);

    if matches!(attr, parse::FieldAttr::Ignore) {
        return None;
    }

    let field_ident = &field.ident;
    let field_name = match &attr {
        parse::FieldAttr::Alias(alias) => quote!(::nova_reflect::FieldName::from(#alias)),
        _ if is_named => quote!(::nova_reflect::FieldName::from(stringify!(#field_ident))),
        _ => quote!(::nova_reflect::FieldName::from(#index)),
    };

    let field_type = &field.ty;
    let field_vis = reflect_visibility::build(&field.vis);
    let field_meta = reflect_meta::build(&field.attrs);

    Some(quote! {
        ::nova_reflect::field()
            .name(#field_name)
            .ty(::nova_reflect::type_of!(#field_type))
            .visibility(#field_vis)
            .meta(#field_meta)
            .build()
    })
}
