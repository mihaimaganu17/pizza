use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(ReadMe)]
pub fn derive(input: TokenStream) -> TokenStream {
    let token_stream2 = proc_macro2::TokenStream::from(input);
    let input = syn::parse2::<syn::DeriveInput>(token_stream2).expect("Cannot parse input");

    let name = input.ident;

    let tokens = quote! {
        impl read_me::Primitive for #name {
            fn read(data: &[u8]) -> Result<Self, ReaderError> {
                // Test run
                Err(ReaderError::InsufficientBytes(0, 0))
            }
        }
    };


    /*
    match input.data {
        Data(data_struct) => {
        }
    }
    */

    println!("{:#?}", input);
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
