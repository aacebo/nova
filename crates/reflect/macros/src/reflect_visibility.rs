use quote::{ToTokens, quote};

pub fn build(vis: &syn::Visibility) -> proc_macro2::TokenStream {
    match vis {
        syn::Visibility::Inherited => quote!(::nova_reflect::Visibility::Private),
        syn::Visibility::Public(_) => quote! {
            ::nova_reflect::Visibility::Public(
                ::nova_reflect::Public::Full
            )
        },
        syn::Visibility::Restricted(v) => {
            let path = v.path.to_token_stream().to_string();

            match path.as_str() {
                "self" => quote! {
                    ::nova_reflect::Visibility::Public(
                        ::nova_reflect::Public::Type
                    )
                },
                "crate" => quote! {
                    ::nova_reflect::Visibility::Public(
                        ::nova_reflect::Public::Crate
                    )
                },
                "super" => quote! {
                    ::nova_reflect::Visibility::Public(
                        ::nova_reflect::Public::Super
                    )
                },
                other => quote! {
                    ::nova_reflect::Visibility::Public(
                        ::nova_reflect::Public::Mod(#other.to_string())
                    )
                },
            }
        }
    }
}
