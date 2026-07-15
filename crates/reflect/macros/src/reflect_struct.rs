use quote::quote;

use crate::{parse, reflect_field, reflect_generics, reflect_meta, reflect_visibility};

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

    let arms = field_arms(&data.fields);

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
            fn to_value_ref(&self) -> ::nova_reflect::ValueRef<'_> {
                ::nova_reflect::ValueRef::Dynamic(::nova_reflect::DynamicRef::from_object(self))
            }
        }

        impl ::nova_reflect::Object for #name {
            fn field(&self, name: &str) -> ::nova_reflect::ValueRef<'_> {
                #(#arms)*

                ::nova_reflect::ValueRef::Undefined
            }

            fn call(
                &self,
                name: &str,
                args: &[::nova_reflect::ValueRef],
            ) -> ::std::result::Result<::nova_reflect::Value, ::std::string::String> {
                #[allow(unused_imports)]
                use ::nova_reflect::Methods as _;

                self.call_method(name, args)
            }
        }
    }
}

fn field_arms(fields: &syn::Fields) -> Vec<proc_macro2::TokenStream> {
    match fields {
        syn::Fields::Named(named) => named
            .named
            .iter()
            .filter_map(|field| {
                let ident = field.ident.as_ref()?;

                match parse::field_attr(&field.attrs) {
                    parse::FieldAttr::Ignore => None,
                    parse::FieldAttr::Alias(alias) => Some(quote! {
                        if name == #alias {
                            return ::nova_reflect::ToValue::to_value_ref(&self.#ident);
                        }
                    }),
                    parse::FieldAttr::Default => Some(quote! {
                        if name == stringify!(#ident) {
                            return ::nova_reflect::ToValue::to_value_ref(&self.#ident);
                        }
                    }),
                }
            })
            .collect(),
        syn::Fields::Unnamed(unnamed) => unnamed
            .unnamed
            .iter()
            .enumerate()
            .filter_map(|(i, field)| {
                if matches!(parse::field_attr(&field.attrs), parse::FieldAttr::Ignore) {
                    return None;
                }

                let index = syn::Index::from(i);
                let key = i.to_string();

                Some(quote! {
                    if name == #key {
                        return ::nova_reflect::ToValue::to_value_ref(&self.#index);
                    }
                })
            })
            .collect(),
        syn::Fields::Unit => vec![],
    }
}

pub fn attr(item: &syn::ItemStruct) -> proc_macro2::TokenStream {
    let name = &item.ident;
    let ty = build(item);
    let arms = field_arms(&item.fields);

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
            fn to_value_ref(&self) -> ::nova_reflect::ValueRef<'_> {
                ::nova_reflect::ValueRef::Dynamic(::nova_reflect::DynamicRef::from_object(self))
            }
        }

        impl ::nova_reflect::Object for #name {
            fn field(&self, name: &str) -> ::nova_reflect::ValueRef<'_> {
                #(#arms)*

                ::nova_reflect::ValueRef::Undefined
            }

            fn call(
                &self,
                name: &str,
                args: &[::nova_reflect::ValueRef],
            ) -> ::std::result::Result<::nova_reflect::Value, ::std::string::String> {
                #[allow(unused_imports)]
                use ::nova_reflect::Methods as _;

                self.call_method(name, args)
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
            .filter_map(|(i, field)| reflect_field::build(field, i, true))
            .collect::<Vec<_>>(),
        syn::Fields::Unnamed(unnamed_fields) => unnamed_fields
            .unnamed
            .iter()
            .enumerate()
            .filter_map(|(i, field)| reflect_field::build(field, i, false))
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
