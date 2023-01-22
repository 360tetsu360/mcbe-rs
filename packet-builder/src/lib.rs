extern crate proc_macro;
use quote::quote;
use syn::{
    parse_macro_input, Attribute, Data, DeriveInput, Error, Fields, FieldsNamed, Ident, Lit,
    LitStr, Meta, MetaList, MetaNameValue, NestedMeta,
};

#[proc_macro_derive(Den, attributes(den))]
pub fn derive_den(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = parse_macro_input!(input as DeriveInput);
    let struct_name = item.ident;
    let fields = extract_struct_fields(&item.data);
    let decode = fields.named.iter().map(|field| {
        let ty = &field.ty;
        let ident = &field.ident;
        let attrs = &field.attrs;
        let attr_each = parse_attr_each(attrs);
        match attr_each {
            Some(AttributeType::InvalidKey(meta)) => {
                Error::new_spanned(meta, "expected `den(with = `T : DenWith<R>`)`")
                    .to_compile_error()
            }
            Some(AttributeType::DenWith(with)) => {
                let den_with = Ident::new(&with.value(), with.span());
                quote! {
                    #ident : <#den_with as byte_util::DenWith<#ty>>::decode(bytes)?
                }
            }
            None => {
                quote! {
                    #ident : byte_util::Den::decode(bytes)?
                }
            }
        }
    });

    let encode = fields.named.iter().map(|field| {
        let ty = &field.ty;
        let ident = &field.ident;
        let attrs = &field.attrs;
        let attr_each = parse_attr_each(attrs);

        match attr_each {
            Some(AttributeType::InvalidKey(meta)) => {
                Error::new_spanned(meta, "expected `den(with = `T : DenWith<R>`)`")
                    .to_compile_error()
            }
            Some(AttributeType::DenWith(with)) => {
                let den_with = Ident::new(&with.value(), with.span());
                quote! {
                    <#den_with as byte_util::DenWith<#ty>>::encode(&self.#ident, bytes)?;
                }
            }
            None => {
                quote! {
                    byte_util::Den::encode(&self.#ident,bytes)?;
                }
            }
        }
    });

    let len = fields.named.iter().map(|field| {
        let ty = &field.ty;
        let ident = &field.ident;
        let attrs = &field.attrs;
        let attr_each = parse_attr_each(attrs);
        match attr_each {
            Some(AttributeType::InvalidKey(meta)) => {
                Error::new_spanned(meta, "expected `den(with = `T : DenWith<R>`)`")
                    .to_compile_error()
            }
            Some(AttributeType::DenWith(with)) => {
                let den_with = Ident::new(&with.value(), with.span());
                quote! {
                    + <#den_with as byte_util::DenWith<#ty>>::size(&self.#ident)
                }
            }
            None => {
                quote! {
                    + byte_util::Den::size(&self.#ident)
                }
            }
        }
    });
    let gen = quote! {
        impl byte_util::Den for #struct_name {
            fn decode(bytes: &mut std::io::Cursor<&[u8]>) -> std::io::Result<Self> {
                Ok(Self {
                    #(#decode),*
                })
            }
            fn encode(&self, bytes : &mut std::io::Cursor<Vec<u8>>) -> std::io::Result<()> {
                #(#encode)*
                Ok(())
            }
            fn size(&self) -> usize {
                let size = 0
                    #(#len)*
                ;
                size
            }
        }
    };

    gen.into()
}

fn extract_struct_fields(data: &Data) -> &FieldsNamed {
    match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => fields,
            _ => panic!("invalid fields"),
        },
        _ => panic!("invalid data"),
    }
}

enum AttributeType {
    DenWith(LitStr),
    InvalidKey(Meta),
}

fn parse_attr_each(attrs: &[Attribute]) -> std::option::Option<AttributeType> {
    attrs.iter().find_map(|attr| match attr.parse_meta() {
        Ok(meta) => match meta {
            Meta::List(MetaList {
                ref path,
                paren_token: _,
                ref nested,
            }) => {
                (path.get_ident()? == "den").then_some(())?;

                if let NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                    path,
                    eq_token: _,
                    lit: Lit::Str(ref litstr),
                })) = nested.first()?
                {
                    if *path.get_ident()? == "with" {
                        Some(AttributeType::DenWith(litstr.clone()))
                    } else {
                        Some(AttributeType::InvalidKey(meta))
                    }
                } else {
                    None
                }
            }
            _ => None,
        },
        _ => None,
    })
}
