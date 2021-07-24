extern crate proc_macro;
use proc_macro2::Span;
use proc_macro::{TokenStream};
use quote::{ToTokens, quote};
use syn::PathSegment;
use syn::parse_macro_input;
use syn::Expr;


/// Note that this isn't actually used because I fear it might cause very long compilation times
#[proc_macro]
pub fn read_csr(item: TokenStream) -> TokenStream {
    let args: Expr = parse_macro_input!(item as Expr);
    
    let csr;
    match args {
        Expr::Path(path) => {
            let path_vec: Vec<String> = path.path.segments.iter().map(|s| s.clone().into_token_stream().to_string()).collect();
            csr = path_vec.join("::");
        },
        _ => panic!("Only plain identifiers are allowed, not {:?}", args),
    }
    
    let s = syn::LitStr::new(&format!("csrr $0, {}", csr), Span::call_site());
    
    (quote! {
        {
            let value: usize;
            unsafe { llvm_asm!(#s : "=r"(value) ::: "volatile") };
            value
        }
    }).into()
}
