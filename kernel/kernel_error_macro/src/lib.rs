/// thiserror and snafu didn't match my needs exactly
/// so I wrote an alternative

#[macro_use]
extern crate proc_macro_error;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::ToTokens;
use syn::{
    parse_macro_input,
    token::{Enum, Paren},
    Data, DeriveInput, Expr, ExprPath, Fields, Path,
};

#[proc_macro_error]
#[proc_macro_derive(KError)]
pub fn derive_kerror(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let enum_data = match input.data {
        Data::Enum(data) => data,
        Data::Struct(data) => abort!(data.struct_token.span, "Can only derive KError on enums!"),
        Data::Union(data) => abort!(data.union_token.span, "Can only derive KError on enums!"),
    };

    let enum_ident = input.ident;

    let variant_impl: Vec<TokenStream2> = enum_data
        .variants
        .iter()
        .filter(|s| match &s.fields {
            Fields::Unnamed(e) => e.unnamed.len() != 0,
            _ => false,
        })
        .map(|variant| {
            let variant_ident = variant.ident.clone();
            let data_type = match &variant.fields {
                Fields::Unnamed(unnamed) => {
                    if unnamed.unnamed.len() == 0 {
                        unreachable!()
                    } else if unnamed.unnamed.len() == 1 {
                        unnamed.unnamed.first().unwrap().ty.clone()
                    } else {
                        syn::TypeTuple {
                            paren_token: Paren {
                                span: unnamed.paren_token.span,
                            },
                            elems: unnamed
                                .unnamed
                                .iter()
                                .map(|field| field.ty.clone())
                                .collect(),
                        }
                        .into()
                    }
                }
                _ => unreachable!(),
            };
            let data_pattern = match &variant.fields {
                Fields::Unnamed(unnamed) => {
                    if unnamed.unnamed.len() == 0 {
                        unreachable!()
                    } else if unnamed.unnamed.len() == 1 {
                        quote! { (value) }.into()
                    } else {
                        syn::ExprTuple {
                            paren_token: Paren {
                                span: unnamed.paren_token.span,
                            },
                            elems: (0..unnamed.unnamed.len())
                                .map(|number| {
                                    Ident::new(
                                        &("i_".to_string() + &number.to_string()),
                                        Span::call_site(),
                                    )
                                })
                                .map::<Expr, _>(|ident| {
                                    Expr::Path(ExprPath {
                                        path: Path::from(ident),
                                        qself: None,
                                        attrs: Vec::new(),
                                    })
                                    .into()
                                })
                                .collect(),
                            attrs: Vec::new(),
                        }
                        .into_token_stream()
                    }
                }
                _ => unreachable!(),
            };
            quote! {
                impl ::core::convert::From<#data_type> for #enum_ident {
                    fn from(value: #data_type) -> #enum_ident {
                        let #data_pattern = value;
                        #enum_ident :: #variant_ident #data_pattern
                    }
                }
            }
        })
        .collect();

    quote! { #(#variant_impl)* }.into()
}
