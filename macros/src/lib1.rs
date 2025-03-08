#![feature(inherent_associated_types)]
use proc_macro::TokenStream;
mod struct_def;
mod tuple_iter;
mod fn_def;
mod extra_iter;

extern crate proc_macro;

#[proc_macro_attribute]
pub fn struct_reflect(_attr: TokenStream, items: TokenStream) -> TokenStream {

    let struct_defn = struct_def::StructDef::analyze_stream(items.clone());

    println!("{:#?}", struct_defn);
    items
}