use quote::{ToTokens, quote};

pub fn build(vis: &syn::Visibility) -> proc_macro2::TokenStream {
    match vis {
        syn::Visibility::Inherited => quote!(::ayr_reflect::Visibility::Private),
        syn::Visibility::Public(_) => quote! {
            ::ayr_reflect::Visibility::Public(
                ::ayr_reflect::Public::Full
            )
        },
        syn::Visibility::Restricted(v) => {
            let path = v.path.to_token_stream().to_string();

            match path.as_str() {
                "self" => quote! {
                    ::ayr_reflect::Visibility::Public(
                        ::ayr_reflect::Public::Type
                    )
                },
                "crate" => quote! {
                    ::ayr_reflect::Visibility::Public(
                        ::ayr_reflect::Public::Crate
                    )
                },
                "super" => quote! {
                    ::ayr_reflect::Visibility::Public(
                        ::ayr_reflect::Public::Super
                    )
                },
                other => quote! {
                    ::ayr_reflect::Visibility::Public(
                        ::ayr_reflect::Public::Mod(#other.to_string())
                    )
                },
            }
        }
    }
}
