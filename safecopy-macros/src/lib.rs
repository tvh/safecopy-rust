extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

#[proc_macro_derive(SafeCopy)]
pub fn derive_safecopy(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;

    let (parse_body, write_body) = match ast.data {
        syn::Data::Struct(data_struct) => match data_struct.fields {
            syn::Fields::Named(fields_named) => {
                let field_names = fields_named
                    .named
                    .iter()
                    .map(|f| &f.ident)
                    .collect::<Vec<_>>();
                (
                    quote! {
                        Ok(#name {
                            #(#field_names: safecopy::safe_parse(reader)?,)*
                        })
                    },
                    quote! {
                        #(safecopy::safe_write(writer, &value.#field_names)?;)*
                        Ok(())
                    },
                )
            }
            syn::Fields::Unnamed(fields_unnamed) => {
                let field_names = (0..fields_unnamed.unnamed.len())
                    .map(|i| syn::Index::from(i))
                    .collect::<Vec<_>>();
                let parse_calls = field_names
                    .iter()
                    .map(|_i| quote! { safecopy::safe_parse(reader)? });
                (
                quote! {
                    Ok(#name(
                        #(#parse_calls,)*
                    ))
                },
                quote! {
                    #(safecopy::safe_write(writer, &value.#field_names)?;)*
                    Ok(())
                },
            )},
            syn::Fields::Unit => (
                quote! {
                    Ok(#name)
                },
                quote! {
                    Ok(())
                },
            ),
        },
        syn::Data::Enum(data_enum) => (
            quote! {
                todo!("Enum: parse")
            },
            quote! {
                todo!("Enum: write")
            },
        ),
        syn::Data::Union(_data_union) => {
            panic!("Unions are not supported");
        }
    };

    let gen = quote! {
        use bincode::*;
        use safecopy::*;
        use std::io::{Read, Write};

        impl safecopy::SafeCopy for #name {
            type K = Base; // FIXME: Make this configurable
            const VERSION: i32 = 0; // FIXME: Make this configurable

            fn parse_unsafe<R: Read>(reader: &mut R) -> Result<Self> {
                #parse_body
            }

            fn write_unsafe<W: Write>(writer: &mut W, value: &Self) -> Result<()> {
                #write_body
            }
        }
    };
    gen.into()
}
