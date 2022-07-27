use proc_macro::{TokenStream};
use syn;
use quote::{self, ToTokens};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ts = syn::parse_macro_input!(input as syn::DeriveInput);
    let ident_name = ts.ident.to_token_stream();

    quote::quote! {
        impl #ident_name {
            fn builder() {
                println!("create builder fn successfully!")
            }
        }
    }.into()
}
