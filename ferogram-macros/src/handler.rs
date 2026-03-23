// Copyright 2024-2026 - Andriel Ferreira
//
// Licensed under the MIT license <LICENSE or https://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

//! Handler builder macros.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use serde::Serialize;
use syn::{Expr, ItemFn};

/// Build a new handler.
pub fn new_handler(update_type: UpdateType, filters: Option<Expr>, input: ItemFn) -> TokenStream {
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = input;

    let name = sig.ident.clone();
    let inputs = sig.inputs.clone();
    let filter = filters.map_or(quote! { ::ferogram::filter::always }, |expr| {
        transform_filter(&expr)
    });

    let handler = match update_type {
        UpdateType::NewMessage => quote! { ::ferogram::handler::new_message },
        UpdateType::MessageEdited => quote! { ::ferogram::handler::message_edited },
        UpdateType::MessageDeleted => quote! { ::ferogram::handler::message_deleted },
        UpdateType::CallbackQuery => quote! { ::ferogram::handler::callback_query },
        UpdateType::InlineQuery => quote! { ::ferogram::handler::inline_query },
        UpdateType::InlineSend => quote! { ::ferogram::handler::inline_send },
        UpdateType::Raw => quote! { ::ferogram::handler::new_raw },
    };

    quote! {
        #(#attrs)*
        #vis fn #name() -> ::ferogram::Handler {

            #handler(#filter)
                .then(|#inputs| async move {
                    #block
                })
        }
    }
    .into()
}

/// Recursively transforms macro filters syntax into ferogram actual filters syntax.
fn transform_filter(expr: &Expr) -> TokenStream2 {
    match expr {
        // Handle parenthereses `(A || B) && C`.
        Expr::Paren(paren) => {
            let inner = transform_filter(&paren.expr);
            quote! { (#inner) }
        }
        // Handle unary operations (!).
        Expr::Unary(unary) => {
            // Recursively transform the inner expression first
            let inner = transform_filter(&unary.expr);

            match unary.op {
                // Convert `!private` -> `private.not()`
                syn::UnOp::Not(_) => quote! { #inner.not() },
                // Fallback
                _ => quote! { #unary.op #inner },
            }
        }
        // Handle binary operations (&&, ||).
        Expr::Binary(bin) => {
            // Recursively transform the left and right sides.
            let left = transform_filter(&bin.left);
            let right = transform_filter(&bin.right);

            match bin.op {
                // Convert `left && right` -> `left.and(right)`.
                syn::BinOp::And(_) => quote! { #left.and(#right) },
                // Convert `left || right` -> `left.or(right)`.
                syn::BinOp::Or(_) => quote! { #left.or(#right) },
                // Fallback
                _ => quote! { #left #bin.op #right },
            }
        }
        _ => quote! {#expr },
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub enum UpdateType {
    NewMessage,
    MessageEdited,
    MessageDeleted,
    CallbackQuery,
    InlineQuery,
    InlineSend,
    #[default]
    Raw,
}
