use proc_macro::TokenStream;
use quote::quote;
use syn::{Meta, Data, Fields, punctuated::Punctuated, Ident, Token, MetaNameValue, GenericParam, Type};
use proc_macro2::TokenStream as TokenStream2;
use syn::parse::Parser;

#[proc_macro_derive(ReadMe, attributes(from, handler, option))]
// TODO: Handle option
pub fn derive(input: TokenStream) -> TokenStream {
    let token_stream2 = TokenStream2::from(input);
    let input = syn::parse2::<syn::DeriveInput>(token_stream2).expect("Cannot parse input");

    let name = input.ident;

    // Token stream containing expression that use the reader to read each value from the structure
    let mut read_expr_tokens = TokenStream2::new();
    // Token stream containing instantiation for each field in the structure
    let mut instantiate_tokens = TokenStream2::new();
    // Token stream containing expressions which compute the cumulative size of the structure based
    // on the sizes of each of the containing fields.
    let mut size_on_disk_tokens = TokenStream2::new();
    // Token stream containing a generic from the structure stored as a single ident
    let mut generic_ident = TokenStream2::new();
    // Token stream containing the generic and the traits it has to implement
    let mut generic_traits = TokenStream2::new();

    for (i, generic_param) in input.generics.params.iter().enumerate() {
        match generic_param {
            GenericParam::Type(type_param) => {
                let ident = &type_param.ident;
                generic_ident.extend(quote! { #ident });
                generic_traits.extend(quote! { #type_param });
            }
            _ => unimplemented!("Lifetime and Const generic parameters are not supported"),
        }
    }

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

                        let array_size = TokenStream2::new();

                        let (read_type_expr, size_expr) = match field.ty {
                            _ =>{
                                let type_ident = field.ty;
                                (
                                    quote! {
                                        let #ident = reader.read::<#type_ident>()?;
                                    },
                                    quote! {
                                        self.#ident.size_on_disk()
                                    },
                                )
                            }
                        };

                        read_expr_tokens.extend(read_type_expr);
                        instantiate_tokens.extend(quote! {
                            #ident,
                        });
                        size_on_disk_tokens.extend(quote! { #size_expr + })
                    }
                }
                _ => unimplemented!("Unamed and unit fields are not implemented")
            }
        }
        _ => unimplemented!("Current procedural macro is not implemented for `Enum` and `Union`"),
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
    }

    let name_with_gen = if !generic_ident.is_empty() {
        quote! { #name<#generic_ident> }
    } else {
        quote! { #name }
    };

    let impl_generic = if !generic_traits.is_empty() {
        quote! { <#generic_traits> }
    } else {
        quote! {}
    };

    // Construct the `Primitive` trait implementation for this structure.
    let tokens = quote! {
        impl #impl_generic read_me::Primitive for #name_with_gen {
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
