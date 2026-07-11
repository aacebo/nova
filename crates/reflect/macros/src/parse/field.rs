pub enum FieldAttr {
    Default,
    Alias(String),
    Ignore,
}

pub fn field_attr(attrs: &[syn::Attribute]) -> FieldAttr {
    for attr in attrs.iter().filter(|a| a.path().is_ident("field")) {
        if let Ok(lit) = attr.parse_args::<syn::LitStr>() {
            return FieldAttr::Alias(lit.value());
        }

        if let Ok(ident) = attr.parse_args::<syn::Ident>()
            && ident == "ignore"
        {
            return FieldAttr::Ignore;
        }
    }

    FieldAttr::Default
}
