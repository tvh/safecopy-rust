extern crate proc_macro;
use proc_macro::TokenStream;
use syn::DeriveInput;
use quote::quote;

#[proc_macro_derive(SafeCopy)]
pub fn derive_safecopy(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;
    let gen = quote! {
        use bincode::*;
        use safecopy::*;
        use std::io::{Read, Write};

        impl safecopy::SafeCopy for #name {
            type K = Base; // FIXME: Make this configurable
            const VERSION: i32 = 0; // FIXME: Make this configurable

            fn parse_unsafe<R: Read>(reader: &mut R) -> Result<Self> {
                todo!()
            }

            fn write_unsafe<W: Write>(writer: &mut W, value: &Self) -> Result<()> {
                todo!()
            }
        }
    };
    gen.into()
}