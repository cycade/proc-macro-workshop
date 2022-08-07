use proc_macro::{TokenStream};
use proc_macro2::{TokenStream as TStream, Ident};
use syn;
use quote::{self, ToTokens, spanned::Spanned};

fn create_stream(stream_series: Vec<TStream>) -> TStream {
    let mut stream = TStream::new();
    stream.extend(stream_series.into_iter());
    stream
}

fn anomy_ident(ident: &TStream) -> Ident {
    let tpl = format!("_{}", ident);
    return syn::Ident::new(&tpl, ident.__span());
}

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let mut stream_pair: Vec<(TStream, TStream, Ident)> = vec![];
    let input_stream = syn::parse_macro_input!(input as syn::DeriveInput);

    match input_stream.data {
        syn::Data::Struct(data) => {
            match data.fields {
                syn::Fields::Named(fields) => {
                    for f in fields.named.into_iter() {
                        let fn_ident = f.ident.unwrap().to_token_stream();
                        let param_ty = f.ty.to_token_stream();
                        let varname = anomy_ident(&fn_ident);
                        stream_pair.push((fn_ident, param_ty, varname));
                    }
                },
                _ => (),
            }
        },
        _ => (),
    }

    let field_stream = create_stream(
        stream_pair.iter().map(|(ident, ty, _varname)| {
            quote::quote! { #ident: #ty, }
        }).collect()
    );

    let op_field_stream = create_stream(
        stream_pair.iter().map(|(_ident, ty, varname)| {
            return quote::quote! { #varname: Option<#ty>, }
        }).collect()
    );

    let default_stream = create_stream(
        stream_pair.iter().map(|(_ident, _ty, varname)| {
            quote::quote! { #varname: None, }
        }).collect()
    );

    let method_stream = create_stream(
        stream_pair.iter().map(|(ident, ty, varname)| {
            quote::quote! {
                impl OptionBuilderT {
                    fn #ident(&mut self, input: #ty) -> &mut OptionBuilderT {
                        self.#varname = Some(input);
                        self
                    }
                }
            }
        }).collect()
    );

    let build_stream = create_stream(
        stream_pair.iter().map(|(ident, _ty, varname)| {
            quote::quote! { #ident: self.#varname.as_ref().unwrap().clone(), }
        }).collect()
    );

    let struct_ident = input_stream.ident.to_token_stream();
    quote::quote! {
        impl #struct_ident {
            fn builder() -> OptionBuilderT {
                OptionBuilderT{..Default::default()}
            }
        }

        pub struct BuilderT {
            #field_stream
        }

        pub struct OptionBuilderT {
            #op_field_stream
        }

        
        impl Default for OptionBuilderT {
            fn default() -> OptionBuilderT {
                OptionBuilderT{
                    #default_stream
                }
            }
        }

        #method_stream

        impl OptionBuilderT {
            fn build(&self) -> Option<BuilderT> {
                return Some(BuilderT{
                    #build_stream
                });
            }
        }
    }.into()
}
