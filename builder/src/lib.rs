use proc_macro::{TokenStream};
use syn;
use quote::{self, ToTokens};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let mut qoute_series: Vec<TokenStream> = vec![];

    let input_stream = syn::parse_macro_input!(input as syn::DeriveInput);
    let struct_ident = input_stream.ident.to_token_stream();
    qoute_series.push(quote::quote! {
        struct B {}
        impl #struct_ident {
            fn builder() -> B {
                B{}
            }
        }
    }.into());

    match input_stream.data {
        syn::Data::Struct(data) => {
            match data.fields {
                syn::Fields::Named(fields) => {
                    for f in fields.named.into_iter() {
                        let fn_ident = f.ident.unwrap().to_token_stream();
                        let param_ty = f.ty.to_token_stream();
                        qoute_series.push(quote::quote! {
                            impl B {
                                fn #fn_ident(&mut self, input: #param_ty) -> () {}
                            }
                        }.into());
                    }
                },
                _ => (),
            }
        },
        _ => (),
    }

    let mut token_stream = proc_macro::TokenStream::new();
    token_stream.extend(qoute_series.into_iter());
    token_stream
}
