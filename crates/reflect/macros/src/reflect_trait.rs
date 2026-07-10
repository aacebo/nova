use quote::quote;

use crate::{reflect_generics, reflect_meta, reflect_visibility};

pub fn attr(meta: proc_macro2::TokenStream, item: &syn::ItemTrait) -> proc_macro2::TokenStream {
    let name = &item.ident;
    let ty = build(meta, item);

    quote! {
        #item

        impl ::ayr_reflect::TypeOf for dyn #name {
            fn type_of() -> ::ayr_reflect::Type {
                ::std::thread_local! {
                    static CACHED: ::std::cell::RefCell<::std::option::Option<::ayr_reflect::Type>>
                        = ::std::cell::RefCell::new(::std::option::Option::None);
                }
                CACHED.with(|c| {
                    let mut guard = c.borrow_mut();
                    if guard.is_none() {
                        *guard = ::std::option::Option::Some(#ty);
                    }
                    guard.as_ref().unwrap().clone()
                })
            }
        }

        impl ::ayr_reflect::ToType for dyn #name {
            fn to_type(&self) -> ::ayr_reflect::Type {
                <dyn #name as ::ayr_reflect::TypeOf>::type_of()
            }
        }
    }
}

pub fn build(meta: proc_macro2::TokenStream, item: &syn::ItemTrait) -> proc_macro2::TokenStream {
    let name = &item.ident;
    let vis = reflect_visibility::build(&item.vis);
    let inner_meta = reflect_meta::build(&item.attrs);
    let generics = reflect_generics::build(&item.generics);
    let methods = item
        .items
        .iter()
        .filter_map(|item| {
            if let syn::TraitItem::Fn(func) = item {
                let fn_name = &func.sig.ident;
                let fn_meta = reflect_meta::build(&func.attrs);
                let fn_is_async = func.sig.asyncness.is_some();

                let fn_return_type = match &func.sig.output {
                    syn::ReturnType::Default => quote!(::ayr_reflect::Type::Void),
                    syn::ReturnType::Type(_, ty) => quote!(::ayr_reflect::type_of!(#ty)),
                };

                let fn_params = func
                    .sig
                    .inputs
                    .iter()
                    .map(|arg| match arg {
                        syn::FnArg::Receiver(recv) => {
                            let mut param_ty = quote! {
                                ::ayr_reflect::ThisType.to_type()
                            };

                            if recv.mutability.is_some() {
                                param_ty = quote!(::ayr_reflect::MutType::new(#param_ty).to_type());
                            }

                            if let syn::Type::Reference(_) = recv.ty.as_ref() {
                                param_ty = quote!(::ayr_reflect::RefType::new(#param_ty).to_type());
                            }

                            quote! {
                                ::ayr_reflect::Param::new(
                                    "self",
                                    #param_ty,
                                )
                            }
                        }
                        syn::FnArg::Typed(typed) => match typed.pat.as_ref() {
                            syn::Pat::Ident(ident) => {
                                let arg_name = &ident.ident;
                                let arg_ty = &typed.ty;
                                let mut param_ty = quote!(::ayr_reflect::type_of!(#arg_ty));

                                if ident.mutability.is_some() {
                                    param_ty =
                                        quote!(::ayr_reflect::MutType::new(#param_ty).to_type());
                                }

                                if let syn::Type::Reference(reference) = typed.ty.as_ref() {
                                    let inner = &reference.elem;
                                    let mut inner_ty = quote!(::ayr_reflect::type_of!(#inner));

                                    if reference.mutability.is_some() {
                                        inner_ty = quote!(
                                            ::ayr_reflect::MutType::new(#inner_ty).to_type()
                                        );
                                    }

                                    param_ty = quote!(
                                        ::ayr_reflect::RefType::new(#inner_ty).to_type()
                                    );
                                }

                                quote! {
                                    ::ayr_reflect::Param::new(
                                        stringify!(#arg_name),
                                        #param_ty,
                                    )
                                }
                            }
                            _ => quote!(),
                        },
                    })
                    .collect::<Vec<_>>();

                return Some(quote! {
                    ::ayr_reflect::Method::new()
                        .name(stringify!(#fn_name))
                        .meta(#fn_meta)
                        .is_async(#fn_is_async)
                        .visibility(::ayr_reflect::Visibility::Public(::ayr_reflect::Public::Full))
                        .params([#(#fn_params,)*])
                        .return_type(#fn_return_type)
                        .build()
                });
            }

            None
        })
        .collect::<Vec<_>>();

    quote! {
        ::ayr_reflect::TraitType::new()
            .path(::ayr_reflect::Path::from(module_path!()))
            .name(stringify!(#name))
            .meta(#meta.merge(&#inner_meta))
            .generics(#generics)
            .visibility(#vis)
            .methods([#(#methods,)*])
            .build()
            .to_type()
    }
}
