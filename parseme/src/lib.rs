use proc_macro::TokenStream;
use quote::quote;
use syn::{Meta, Data, Fields, punctuated::Punctuated, Ident, Token, MetaNameValue};
use proc_macro2::TokenStream as TokenStream2;
use syn::parse::Parser;

#[proc_macro_derive(ReadMe, attributes(from, handler))]
pub fn derive(input: TokenStream) -> TokenStream {
    let token_stream2 = TokenStream2::from(input);
    let input = syn::parse2::<syn::DeriveInput>(token_stream2).expect("Cannot parse input");

    let name = input.ident;

    // Output token stream
    let mut read_expr_tokens = TokenStream2::new();
    // Output token stream
    let mut instantiate_tokens = TokenStream2::new();
    let mut size_on_disk_tokens = TokenStream2::new();

    match input.data {
        Data::Struct(data_struct) => {
            match data_struct.fields {
                Fields::Named(fields) => {
                    // Go through each field
                    for field in fields.named {
                        // Get the ident for the field
                        let ident = if let Some(ident) = field.ident {
                            ident
                        } else {
                            unimplemented!("Tuple fields are not implemented")
                        };

                        let type_ident = field.ty;
                        read_expr_tokens.extend(quote! {
                            let #ident = reader.read::<#type_ident>()?;
                        });
                        instantiate_tokens.extend(quote! {
                            #ident,
                        });
                        size_on_disk_tokens.extend(quote! {
                            self.#ident.size_on_disk() +
                        })
                    }
                }
                _ => unimplemented!("Unamed and unit fields are not implemented")
            }
        }
        Data::Enum(data_enum) => {
            for attr in input.attrs {
                if let Meta::List(list) = attr.meta {
                    if &format!("{}", list.path.segments[0].ident) == "read" {
                        let parser =
                            Punctuated::<MetaNameValue, Token![,]>::parse_separated_nonempty;
                        let args = parser.parse(list.tokens.into()).expect("Failed to parse read arguments");
                    }
                }
            }
        }
        _ => unimplemented!("Current procedural macro is not implemented for `Enum` and `Union`")
    }

    // Construct the `Primitive` trait implementation for this structure.
    let tokens = quote! {
        impl read_me::Primitive for #name {
            fn read(data: &[u8]) -> Result<Self, ReaderError> {
                let mut reader = Reader::from(data);

                #read_expr_tokens
                Ok(#name {
                    #instantiate_tokens
                })
            }

            fn size_on_disk(&self) -> usize {
                #size_on_disk_tokens 0
            }
        }
    };

    tokens.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
