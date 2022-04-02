use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    DeriveInput, Token, Type,
};

struct MyMacroInput {
    items: Vec<Type>,
}

impl Parse for MyMacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            items: Punctuated::<Type, Token![,]>::parse_terminated(input)?
                .into_iter()
                .collect::<Vec<_>>(),
        })
    }
}
#[proc_macro_attribute]
pub fn to_trait(attr: TokenStream, item: TokenStream) -> TokenStream {
    let traits = parse_macro_input!(attr as MyMacroInput).items;
    let item = parse_macro_input!(item as DeriveInput);
    let ident = item.ident.clone();
    quote! {
        #item
        impl ::to_trait::ToTrait for #ident {
            fn cast_to_trait(self, target_type_id: core::any::TypeId) -> Option<Box<dyn ::to_trait::Null>> {
                #(if core::any::TypeId::of::<dyn #traits>() == target_type_id {
                    let b: Box<dyn #traits> = Box::new(self);
                    let b: Box<dyn ::to_trait::Null> = unsafe { Box::from_raw(core::mem::transmute_copy(&Box::into_raw(b))) };
                    return Some(b);
                })*
                return None;
            }
            fn cast_to_trait_ref(&self, target_type_id: core::any::TypeId) -> Option<&dyn ::to_trait::Null> {
                #(if core::any::TypeId::of::<dyn #traits>() == target_type_id {
                    let b: &(dyn #traits) = self;
                    let b: &dyn ::to_trait::Null = unsafe { core::mem::transmute_copy(&b) };
                    return Some(b);
                })*
                return None;
            }
            fn cast_to_trait_mut(&mut self, target_type_id: core::any::TypeId) -> Option<&mut dyn ::to_trait::Null> {
                #(if core::any::TypeId::of::<dyn #traits>() == target_type_id {
                    let b: &mut (dyn #traits) = self;
                    let b: &mut dyn ::to_trait::Null = unsafe { core::mem::transmute_copy(&b) };
                    return Some(b);
                })*
                return None;
            }
        }
    }.into()
}
