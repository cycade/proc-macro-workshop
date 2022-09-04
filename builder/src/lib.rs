use proc_macro::{TokenStream};
use proc_macro2::{TokenStream as TStream, Ident};
use syn::{self};
use quote::{self, ToTokens, spanned::Spanned};

fn create_stream(stream_series: Vec<TStream>) -> TStream {
    let mut stream = TStream::new();
    stream.extend(stream_series.into_iter());
    stream
}

fn anomy_ident(ident: &Ident) -> Ident {
    let tpl = format!("_{}", ident);
    return syn::Ident::new(&tpl, ident.__span());
}

fn is_option_field(f: syn::Field) -> (TStream, bool) {
    return match f.ty {
        syn::Type::Path(t) => {
            let tys = t.path.segments.clone();
            if tys[0].ident.to_string() == "Option" {
                let type_params = tys.first().unwrap();
                let generic_arg = match &type_params.arguments {
                    syn::PathArguments::AngleBracketed(params) => params.args.first().unwrap(),
                    _ => unimplemented!("syn arguments"),
                };
                (generic_arg.to_token_stream(), true)
            } else {
                (t.to_token_stream(), false)
            }
        },
        _ => (TStream::new(), false),
    };
}

// struct_ident, struct_ty, builder_ident, builder_ty, is_option_field
type BuilderTyPair = (Ident, TStream, Ident, TStream, bool);

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let mut stream_pair: Vec<BuilderTyPair> = vec![];
    let input_stream = syn::parse_macro_input!(input as syn::DeriveInput);
    let struct_ident = input_stream.ident.clone();
    
    match input_stream.data {
        syn::Data::Struct(data) => {
            match data.fields {
                syn::Fields::Named(fields) => {
                    for f in fields.named.into_iter() {
                        let field_ident = f.ident.clone().unwrap();
                        let field_ty = f.ty.to_token_stream();
                        let builder_ident = anomy_ident(&field_ident);
                        let (builder_ty, is_option_field) = is_option_field(f.clone());

                        stream_pair.push((field_ident, field_ty, builder_ident, builder_ty, is_option_field));
                    }
                },
                _ => (),
            }
        },
        _ => (),
    }

    let builder_field_stream = create_stream(
        stream_pair.iter().map(|(_, ty, builder_ident, _, _)| {
            return quote::quote! { #builder_ident: Option<#ty>, }
        }).collect()
    );

    let builder_default_stream = create_stream(
        stream_pair.iter().map(|(_, _, builder_ident, _, _)| {
            quote::quote! { #builder_ident: None, }
        }).collect()
    );

    let builder_method_stream = create_stream(
        stream_pair.iter().map(|(struct_ident, ty, builder_ident, builder_ty, is_option_field)| {
            if *is_option_field {
                quote::quote! {
                    impl OptionBuilderT {
                    fn #struct_ident(&mut self, input: #builder_ty) -> &mut OptionBuilderT {
                        self.#builder_ident = Some(Some(input));
                        self
                        }
                    }
                }
            } else {
                quote::quote! {
                    impl OptionBuilderT {
                        fn #struct_ident(&mut self, input: #ty) -> &mut OptionBuilderT {
                            self.#builder_ident = Some(input);
                            self
                        }
                    }
                } 
            }
        }).collect()
    );

    let build_into_stream = create_stream(
        stream_pair.iter().map(|(struct_ident, _, builder_ident, _, is_option_field)| {
            if *is_option_field {
                return quote::quote! {
                    #struct_ident: match &self.#builder_ident {
                        Some(x) => x.clone(),
                        None => None,
                    },
                };
            }
            
            quote::quote! {
                #struct_ident: self.#builder_ident.as_ref().unwrap().clone(),
            }
        }).collect()
    );

    quote::quote! {
        impl #struct_ident {
            fn builder() -> OptionBuilderT {
                OptionBuilderT{..Default::default()}
            }
        }

        pub struct OptionBuilderT {
            #builder_field_stream
        }
        
        impl Default for OptionBuilderT {
            fn default() -> OptionBuilderT {
                OptionBuilderT{
                    #builder_default_stream
                }
            }
        }

        #builder_method_stream

        impl OptionBuilderT {
            fn build(&self) -> Option<#struct_ident> {
                return Some(#struct_ident{
                    #build_into_stream
                });
            }
        }
    }.into()
}
