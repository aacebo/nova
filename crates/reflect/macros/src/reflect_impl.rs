use quote::quote;

use crate::{reflect_generics, reflect_meta};

pub fn build(item: &syn::ItemImpl) -> proc_macro2::TokenStream {
    let impl_for = &item.self_ty;
    let impl_meta = reflect_meta::build(&item.attrs);
    let impl_generics = reflect_generics::build(&item.generics);
    let impl_trait = item.trait_.as_ref().map(|(_, path, _)| quote!(#path));

    match &impl_trait {
        None => quote! {
            ::nova_reflect::implement()
                .path(::nova_reflect::Path::from(module_path!()))
                .ty(::nova_reflect::type_of!(#impl_for))
                .meta(#impl_meta)
                .generics(#impl_generics)
                .build()
        },
        Some(of) => quote! {
            ::nova_reflect::implement()
                .path(::nova_reflect::Path::from(module_path!()))
                .ty(::nova_reflect::type_of!(#impl_for))
                .meta(#impl_meta)
                .generics(#impl_generics)
                .of(::nova_reflect::Path::from(stringify!(#of)))
                .build()
        },
    }
}

pub fn attr(item: &syn::ItemImpl) -> proc_macro2::TokenStream {
    let self_ty = &item.self_ty;
    let members = member_methods(item);
    let dispatch = members.iter().map(|m| m.dispatch()).collect::<Vec<_>>();

    quote! {
        #item

        impl #self_ty {
            pub fn call_method(
                &self,
                name: &str,
                args: &[::nova_reflect::ValueRef],
            ) -> ::std::result::Result<::nova_reflect::Value, ::std::string::String> {
                match name {
                    #(#dispatch)*
                    _ => ::std::result::Result::Err(::std::format!("no method '{}'", name)),
                }
            }
        }
    }
}

struct MemberMethod {
    ident: syn::Ident,
    name: String,
    params: Vec<syn::Type>,
}

impl MemberMethod {
    fn dispatch(&self) -> proc_macro2::TokenStream {
        let ident = &self.ident;
        let name = &self.name;

        let bindings = self.params.iter().enumerate().map(|(i, ty)| {
            let var = quote::format_ident!("a{}", i);
            quote! {
                let #var = <#ty as ::std::convert::TryFrom<::nova_reflect::ValueRef>>::try_from(
                    args.get(#i).cloned().unwrap_or(::nova_reflect::ValueRef::Undefined)
                )?;
            }
        });

        let vars = (0..self.params.len()).map(|i| quote::format_ident!("a{}", i));

        quote! {
            #name => {
                #(#bindings)*
                ::std::result::Result::Ok(::nova_reflect::Value::from(self.#ident(#(#vars),*)))
            }
        }
    }
}

fn member_methods(item: &syn::ItemImpl) -> Vec<MemberMethod> {
    let mut methods = vec![];

    for impl_item in &item.items {
        let syn::ImplItem::Fn(func) = impl_item else {
            continue;
        };

        if !matches!(func.vis, syn::Visibility::Public(_)) {
            continue;
        }

        let mut inputs = func.sig.inputs.iter();

        if !matches!(inputs.next(), Some(syn::FnArg::Receiver(_))) {
            continue;
        }

        let mut params = vec![];
        let mut ok = true;

        for arg in inputs {
            match arg {
                syn::FnArg::Typed(typed) => params.push((*typed.ty).clone()),
                _ => {
                    ok = false;
                    break;
                }
            }
        }

        if !ok {
            continue;
        }

        methods.push(MemberMethod {
            ident: func.sig.ident.clone(),
            name: func.sig.ident.to_string(),
            params,
        });
    }

    methods
}
