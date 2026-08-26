// Copyright 2024-2026 - Andriel Ferreira
//
// Licensed under the MIT license <LICENSE or https://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

//! Handler builder macros.

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{Expr, ItemFn, punctuated::Punctuated, token::Comma};

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

    let handler = handler_fn_for(&update_type);

    quote! {
        #(#attrs)*
        #vis fn #name() -> ::ferogram::Handler {

            #handler(#filter)
                .then(|#inputs| async move {
                    #block
                })
        }

        ::ferogram::discovery::submit! {
            ::ferogram::discovery::HandlerFactory::new(#name)
        }
    }
    .into()
}

/// Build multiple handlers.
pub fn new_multi_handler(entries: Punctuated<Expr, Comma>, input: ItemFn) -> TokenStream {
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = input;

    let name = sig.ident.clone();
    let inputs = sig.inputs.clone();

    let mut handlers = Vec::new();

    for entry in entries {
        let Expr::Call(call) = entry else {
            return syn::Error::new_spanned(
                entry,
                "expected an update type call like `new_message(filter)`",
            )
            .to_compile_error()
            .into();
        };

        let type_ident = match &*call.func {
            Expr::Path(path_expr) => path_expr.path.get_ident(),
            _ => None,
        };
        let Some(type_ident) = type_ident else {
            return syn::Error::new_spanned(&call.func, "expected an update type name")
                .to_compile_error()
                .into();
        };

        let r#type = type_ident.to_string();
        let update_type = match UpdateType::try_from(r#type.as_str()) {
            Ok(t) => t,
            Err(err) => {
                return syn::Error::new_spanned(type_ident, err)
                    .to_compile_error()
                    .into();
            }
        };
        let handler = handler_fn_for(&update_type);

        if call.args.len() > 1 {
            return syn::Error::new_spanned(
                &call.args,
                "expected at most one filter expression per update type",
            )
            .to_compile_error()
            .into();
        }

        let filter = if let Some(filter_expr) = call.args.first() {
            transform_filter(filter_expr)
        } else {
            quote! { ::ferogram::filter::always }
        };

        let fn_name = format_ident!("{}__{}", name, r#type);
        handlers.push(quote! {
            #[allow(non_snake_case)]
            #(#attrs)*
            #vis fn #fn_name() -> ::ferogram::Handler {

                #handler(#filter)
                    .then(|#inputs| async move {
                        #block
                    })
            }

            ::ferogram::discovery::submit! {
                ::ferogram::discovery::HandlerFactory::new(#fn_name)
            }
        });
    }

    if handlers.is_empty() {
        return syn::Error::new(
            Span::call_site(),
            "multi handler requires at least one update type entry",
        )
        .to_compile_error()
        .into();
    }

    quote! {
        #(#handlers)*
    }
    .into()
}

fn handler_fn_for(update_type: &UpdateType) -> TokenStream2 {
    match update_type {
        UpdateType::NewMessage => quote! { ::ferogram::handler::new_message },
        UpdateType::MessageEdited => quote! { ::ferogram::handler::message_edited },
        UpdateType::MessageDeleted => quote! { ::ferogram::handler::message_deleted },
        UpdateType::CallbackQuery => quote! { ::ferogram::handler::callback_query },
        UpdateType::InlineQuery => quote! { ::ferogram::handler::inline_query },
        UpdateType::InlineSend => quote! { ::ferogram::handler::inline_send },
        UpdateType::Raw => quote! { ::ferogram::handler::new_raw },
    }
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

#[derive(Clone, Debug, Default, PartialEq)]
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

impl TryFrom<&str> for UpdateType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "new_message" => Ok(Self::NewMessage),
            "message_edited" => Ok(Self::MessageEdited),
            "message_deleted" => Ok(Self::MessageDeleted),
            "callback_query" => Ok(Self::CallbackQuery),
            "inline_query" => Ok(Self::InlineQuery),
            "inline_send" => Ok(Self::InlineSend),
            "raw" => Err("multi handler doesn't support raw update type".into()),
            _ => Err(format!(
                "unknown update type `{value}`; expected one of: new_message, message_edited, message_deleted, callback_query, inline_query, inline_send"
            )),
        }
    }
}
