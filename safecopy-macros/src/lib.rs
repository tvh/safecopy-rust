extern crate proc_macro;
use std::fmt::format;

use proc_macro::{TokenStream};
use proc_macro2::{Span};
use quote::quote;
use syn::{DeriveInput, Ident};

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
        syn::Data::Enum(data_enum) => {
            let match_arms = data_enum
                .variants
                .iter()
                .enumerate()
                .map(|(i_large,variant)| {
                    let variant_name = &variant.ident;
                    let i: i16 = i_large as i16;
                    match &variant.fields {
                        syn::Fields::Named(fields_named) => todo!("Enum: Named"),
                        syn::Fields::Unnamed(fields_unnamed) => {
                            let field_count = fields_unnamed.unnamed.len();
                            let field_names = (0..field_count)
                                .map(|i| Ident::new(format!("value{}", i).as_str(), Span::call_site()))
                                .collect::<Vec<_>>();
                            
                            quote! {
                                #name::#variant_name(#(#field_names),*) => {
                                    serialize_into(writer, &#i, Infinite)?;
                                    #(serialize_into(writer, #field_names, Infinite)?;)*
                                },
                            }
                        },
                        syn::Fields::Unit => quote! {
                            #name::#variant_name => serialize_into(writer, &#i, Infinite)?,
                        },
                    }
                })
                .collect::<Vec<_>>();
            (
            quote! {
                todo!("Enum: parse")
            },
            quote! {
                match value {
                    #(#match_arms)*
                }
                Ok(())
            },
        )},
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
