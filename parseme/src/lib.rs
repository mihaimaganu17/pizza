use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, Fields};
use proc_macro2::TokenStream as TokenStream2;

#[proc_macro_derive(ReadMe)]
pub fn derive(input: TokenStream) -> TokenStream {
    let token_stream2 = TokenStream2::from(input);
    let input = syn::parse2::<syn::DeriveInput>(token_stream2).expect("Cannot parse input");

    let name = input.ident;

    // Output token stream
    let mut field_tokens = TokenStream2::new();

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
                        field_tokens.extend(quote! { #ident: #type_ident, });
                    }
                }
                _ => unimplemented!("Unamed and unit fields are not implemented")
            }
        }
        _ => unimplemented!("Current procedural macro is not implemented for `Enum` and `Union`")
    }

    let temp_build = quote! {
        #[derive(Debug)]
        struct Builder {
            #field_tokens
        }
    };
    return temp_build.into();

    // Construct the `Primitive` trait implementation for this structure.
    let tokens = quote! {
        impl read_me::Primitive for #name {
            fn read(data: &[u8]) -> Result<Self, ReaderError> {
                // Test run
                Err(ReaderError::InsufficientBytes(0, 0))
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
