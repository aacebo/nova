use quote::quote;

use crate::{reflect_field, reflect_generics, reflect_meta, reflect_visibility};

pub fn derive(input: &syn::DeriveInput, data: &syn::DataStruct) -> proc_macro2::TokenStream {
    let name = &input.ident;
    let ty = build(&syn::ItemStruct {
        attrs: input.attrs.clone(),
        fields: data.fields.clone(),
        generics: input.generics.clone(),
        ident: input.ident.clone(),
        semi_token: data.semi_token,
        struct_token: data.struct_token,
        vis: input.vis.clone(),
    });

    let fields = match &data.fields {
        syn::Fields::Named(named_fields) => named_fields
            .named
            .iter()
            .map(|field| {
                let field_ident = &field.ident;
                quote!(#field_ident)
            })
            .collect::<Vec<_>>(),
        syn::Fields::Unnamed(unnamed_fields) => unnamed_fields
            .unnamed
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let field_ident = syn::Member::Unnamed(syn::Index {
                    index: i as u32,
                    span: proc_macro2::Span::call_site(),
                });

                quote!(#field_ident)
            })
            .collect::<Vec<_>>(),
        syn::Fields::Unit => vec![],
    };

    quote! {
        impl ::nova_reflect::TypeOf for #name {
            fn type_of() -> ::nova_reflect::Type {
                ::std::thread_local! {
                    static CACHED: ::std::cell::RefCell<::std::option::Option<::nova_reflect::Type>>
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

        impl ::nova_reflect::ToType for #name {
            fn to_type(&self) -> ::nova_reflect::Type {
                <Self as ::nova_reflect::TypeOf>::type_of()
            }
        }

        impl ::nova_reflect::ToValue for #name {
            fn to_value(&self) -> ::nova_reflect::Value<'_> {
                ::nova_reflect::Value::Dynamic(::nova_reflect::Dynamic::from_object(self))
            }
        }

        impl ::nova_reflect::Object for #name {
            fn field(&self, name: &::nova_reflect::FieldName) -> ::nova_reflect::Value<'_> {
                #(
                    if name == stringify!(#fields) {
                        return ::nova_reflect::ToValue::to_value(&self.#fields);
                    }
                )*

                ::nova_reflect::Value::Null
            }
        }
    }
}

pub fn attr(item: &syn::ItemStruct) -> proc_macro2::TokenStream {
    let name = &item.ident;
    let ty = build(item);
    let fields = match &item.fields {
        syn::Fields::Named(named_fields) => named_fields
            .named
            .iter()
            .map(|field| {
                let field_ident = &field.ident;
                quote!(#field_ident)
            })
            .collect::<Vec<_>>(),
        syn::Fields::Unnamed(unnamed_fields) => unnamed_fields
            .unnamed
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let field_ident = syn::Member::Unnamed(syn::Index {
                    index: i as u32,
                    span: proc_macro2::Span::call_site(),
                });

                quote!(#field_ident)
            })
            .collect::<Vec<_>>(),
        syn::Fields::Unit => vec![],
    };

    quote! {
        impl ::nova_reflect::TypeOf for #name {
            fn type_of() -> ::nova_reflect::Type {
                ::std::thread_local! {
                    static CACHED: ::std::cell::RefCell<::std::option::Option<::nova_reflect::Type>>
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

        impl ::nova_reflect::ToType for #name {
            fn to_type(&self) -> ::nova_reflect::Type {
                <Self as ::nova_reflect::TypeOf>::type_of()
            }
        }

        impl ::nova_reflect::ToValue for #name {
            fn to_value(&self) -> ::nova_reflect::Value<'_> {
                ::nova_reflect::Value::Dynamic(::nova_reflect::Dynamic::from_object(self))
            }
        }

        impl ::nova_reflect::Object for #name {
            fn field(&self, name: &::nova_reflect::FieldName) -> ::nova_reflect::Value<'_> {
                #(
                    if name == stringify!(#fields) {
                        return ::nova_reflect::ToValue::to_value(&self.#fields);
                    }
                )*

                ::nova_reflect::Value::Null
            }
        }
    }
}

pub fn build(item: &syn::ItemStruct) -> proc_macro2::TokenStream {
    let name = &item.ident;
    let vis = reflect_visibility::build(&item.vis);
    let meta = reflect_meta::build(&item.attrs);
    let generics = reflect_generics::build(&item.generics);
    let layout = match &item.fields {
        syn::Fields::Named(_) => quote!(::nova_reflect::Layout::Key),
        syn::Fields::Unnamed(_) => quote!(::nova_reflect::Layout::Index),
        syn::Fields::Unit => quote!(::nova_reflect::Layout::Unit),
    };

    let fields = match &item.fields {
        syn::Fields::Named(named_fields) => named_fields
            .named
            .iter()
            .enumerate()
            .map(|(i, field)| reflect_field::build(field, i, true))
            .collect::<Vec<_>>(),
        syn::Fields::Unnamed(unnamed_fields) => unnamed_fields
            .unnamed
            .iter()
            .enumerate()
            .map(|(i, field)| reflect_field::build(field, i, false))
            .collect::<Vec<_>>(),
        syn::Fields::Unit => vec![],
    };

    quote! {
        ::nova_reflect::struct_type()
            .path(::nova_reflect::Path::from(module_path!()))
            .name(stringify!(#name))
            .visibility(#vis)
            .meta(#meta)
            .generics(#generics)
            .fields(
                ::nova_reflect::fields()
                    .layout(#layout)
                    .fields([#(#fields,)*])
                    .build()
            )
            .build()
            .to_type()
    }
}
