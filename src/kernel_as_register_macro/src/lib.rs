extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{parse_macro_input, visit_mut::VisitMut, Data, DeriveInput, Variant};

#[proc_macro_derive(AsRegister)]
pub fn derive_as_register(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let mut enum_data = if let Data::Enum(e) = input.data {
        e
    } else {
        return quote! { compile_error!("Item for AsRegister must be enum") }.into();
    };

    let mut struc = AsRegisterStruct::new();
    struc.visit_data_enum_mut(&mut enum_data);

    let ident = input.ident;

    let recursive_variant_count = &struc.recursive_variant_count;
    let as_register_code = &struc.as_register_code;
    let from_register_code = &struc.from_register_code;
    quote! {
        impl AsRegister for #ident {
            fn recursive_variant_count() -> usize {
                #(#recursive_variant_count)+* + 0
            }
            fn as_register(&self) -> (usize, ::smallvec::SmallVec<[usize; 2]>) {
                match self {
                    #as_register_code
                }
            }
            fn from_register(variant_extra: &(usize, &[usize])) -> Self {
                #from_register_code
                panic!("Register is out of bounds!")
            }
        }
    }
    .into()
}

#[derive(Debug, Default)]
struct AsRegisterStruct {
    member_variants: Vec<()>,
    recursive_variant_count: Vec<TokenStream2>,
    as_register_code: TokenStream2,
    from_register_code: TokenStream2,
}

impl AsRegisterStruct {
    fn new() -> AsRegisterStruct {
        AsRegisterStruct {
            recursive_variant_count: vec![quote! { 0 }],
            ..Default::default()
        }
    }
}

impl VisitMut for AsRegisterStruct {
    fn visit_variant_mut(&mut self, variant: &mut Variant) {
        let fields = variant_fields_get_fields(&variant.fields);
        let variant_count_for_this_type = if fields.len() == 0 {
            quote! {
                1
            }
        } else {
            let mut middle = quote! { 1 };
            for i in fields.iter() {
                let type_of_field = &i.ty;

                middle.extend(quote! {
                    * #type_of_field ::recursive_variant_count()
                })
            }
            middle
        };

        let ident = &variant.ident;

        let recursive_variant_count = &mut self.recursive_variant_count;

        let from_register_creating_code = if fields.len() == 0 {
            quote! {
                Self::#ident
            }
        } else if fields.len() == 1 {
            let t = &fields.iter().next().unwrap().ty;
            quote! {
                Self::#ident (#t ::from_register(&(variant_here, variant_extra.1)))
            }
        } else {
            panic!("aaa panic")
        };

        self.from_register_code.extend(quote! {
            if variant_extra.0 < (#(#recursive_variant_count)+* + #variant_count_for_this_type) {
                let variant_here = #(#recursive_variant_count)+* - variant_extra.0;
                return #from_register_creating_code;
            }
        });

        if fields.len() == 0 {
            self.as_register_code.extend(quote! {
                Self::#ident => {
                    ( (#(#recursive_variant_count)+*), ::smallvec::SmallVec::new())
                }
            })
        } else if fields.len() == 1 {
            self.as_register_code.extend(quote! {
                Self::#ident (e) => {
                    let variants_before_this = #(#recursive_variant_count)+*;
                    let (n, extra) = e.as_register();
                    (n + variants_before_this, extra)
                }
            })
        } else {
            panic!("aaa panic")
        };

        recursive_variant_count.push(variant_count_for_this_type)
    }
}

use syn::punctuated::Punctuated;
fn variant_fields_get_fields(
    f: &syn::Fields,
) -> WeakCow<Punctuated<syn::Field, syn::token::Comma>> {
    use syn::Fields::*;
    match f {
        Named(a) => WeakCow::Borrowed(&a.named),
        Unnamed(a) => WeakCow::Borrowed(&a.unnamed),
        Unit => WeakCow::Owned(Punctuated::new()),
    }
}

#[proc_macro]
pub fn generate_extra_data_structs(_input: TokenStream) -> TokenStream {
    let mut stream = TokenStream::new();
    for i in 0..4 {
        let fields = vec![quote! {usize}].into_iter().cycle().take(i);
        let identifier = Ident::new(&format!("ExtraData{}", i), Span::call_site());
        let stream_here: TokenStream = quote! {
            struct #identifier (#(#fields),*);


        }
        .into();
        stream.extend(stream_here);
    }
    stream
}

use core::ops::Deref;
#[allow(dead_code)]
enum WeakCow<'a, T> {
    Owned(T),
    Borrowed(&'a T),
}

impl<'a, T> Deref for WeakCow<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        use WeakCow::*;
        match self {
            Owned(t) => &t,
            Borrowed(t) => t,
        }
    }
}

#[allow(dead_code)]
enum WeakCowMut<'a, T> {
    Owned(T),
    Borrowed(&'a mut T),
}

impl<'a, T> Deref for WeakCowMut<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        use WeakCowMut::*;
        match self {
            Owned(t) => &t,
            Borrowed(t) => t,
        }
    }
}
