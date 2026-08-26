// Copyright 2024-2026 - Andriel Ferreira
//
// Licensed under the MIT license <LICENSE or https://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

//! General helper macros exports.

mod handler;

use proc_macro::TokenStream;
use syn::{Expr, ItemFn, parse::Parser, punctuated::Punctuated, token::Comma};

use crate::handler::{UpdateType, new_handler, new_multi_handler};

/// Build a new `NewMessage` handler.
///
/// Note: `always` is the default filter; it will be used if no filter is specified.
///
/// # Examples
///
/// ```ignore
/// use ferogram::prelude::*;
/// #[handler::new_message(command("/start :id?"))]
/// async fn start(message: Message) -> Result<()> {
///     Ok(())
/// }
/// ```
#[proc_macro_attribute]
pub fn new_message(attr: TokenStream, input: TokenStream) -> TokenStream {
    let filters = syn::parse::<Expr>(attr).ok();
    let input = syn::parse_macro_input!(input as ItemFn);

    new_handler(UpdateType::NewMessage, filters, input)
}

/// Build a new `MessageEdited` handler.
///
/// Note: `always` is the default filter; it will be used if no filter is specified.
///
/// # Examples
///
/// ```ignore
/// use ferogram::prelude::*;
/// #[handler::message_edited(command("/start :id?"))]
/// async fn start(message: Message) -> Result<()> {
///     Ok(())
/// }
/// ```
#[proc_macro_attribute]
pub fn message_edited(attr: TokenStream, input: TokenStream) -> TokenStream {
    let filters = syn::parse::<Expr>(attr).ok();
    let input = syn::parse_macro_input!(input as ItemFn);

    new_handler(UpdateType::MessageEdited, filters, input)
}

/// Build a new `MessageDeleted` handler.
///
/// Note: `always` is the default filter; it will be used if no filter is specified.
///
/// # Examples
///
/// ```ignore
/// use ferogram::prelude::*;
/// #[handler::message_deleted]
/// async fn start(message: Message) -> Result<()> {
///     Ok(())
/// }
/// ```
#[proc_macro_attribute]
pub fn message_deleted(attr: TokenStream, input: TokenStream) -> TokenStream {
    let filters = syn::parse::<Expr>(attr).ok();
    let input = syn::parse_macro_input!(input as ItemFn);

    new_handler(UpdateType::MessageDeleted, filters, input)
}

/// Build a new `CallbackQuery` handler.
///
/// Note: `always` is the default filter; it will be used if no filter is specified.
///
/// # Examples
///
/// ```ignore
/// use ferogram::prelude::*;
/// #[handler::callback_query(command("start :id?"))]
/// async fn start(query: CallbackQuery) -> Result<()> {
///     Ok(())
/// }
/// ```
#[proc_macro_attribute]
pub fn callback_query(attr: TokenStream, input: TokenStream) -> TokenStream {
    let filters = syn::parse::<Expr>(attr).ok();
    let input = syn::parse_macro_input!(input as ItemFn);

    new_handler(UpdateType::CallbackQuery, filters, input)
}

/// Build a new `InlineQuery` handler.
///
/// Note: `always` is the default filter; it will be used if no filter is specified.
///
/// # Examples
///
/// ```ignore
/// use ferogram::prelude::*;
/// #[handler::inline_query(command("images *query?"))]
/// async fn start(query: InlineQuery) -> Result<()> {
///     Ok(())
/// }
/// ```
#[proc_macro_attribute]
pub fn inline_query(attr: TokenStream, input: TokenStream) -> TokenStream {
    let filters = syn::parse::<Expr>(attr).ok();
    let input = syn::parse_macro_input!(input as ItemFn);

    new_handler(UpdateType::InlineQuery, filters, input)
}

/// Build a new `InlineSend` handler.
///
/// Note: `always` is the default filter; it will be used if no filter is specified.
///
/// # Examples
///
/// ```ignore
/// use ferogram::prelude::*;
/// #[handler::inline_send(command("images *query?"))]
/// async fn start(query: InlineQuery) -> Result<()> {
///     Ok(())
/// }
/// ```
#[proc_macro_attribute]
pub fn inline_send(attr: TokenStream, input: TokenStream) -> TokenStream {
    let filters = syn::parse::<Expr>(attr).ok();
    let input = syn::parse_macro_input!(input as ItemFn);

    new_handler(UpdateType::InlineSend, filters, input)
}

/// Build multiple handlers from a single function, one per update type entry.
///
/// Each entry has the form `update_type(filter)` where `update_type` is one of
/// `new_message`, `message_edited`, `message_deleted`, `callback_query`,
/// `inline_query`, or `inline_send`.
///
/// The function body is shared across all generated handlers. Use parameter
/// types that are injected by every listed update type - `Context` is always
/// injected and lets you access the specific update inside the body.
///
/// # Examples
///
/// ```ignore
/// use ferogram::prelude::*;
/// #[handler::multi(
///     new_message(command("/start")),
///     callback_query(data("start")),
/// )]
/// async fn start(ctx: Context) -> Result<()> {
///     Ok(())
/// }
/// ```
#[proc_macro_attribute]
pub fn multi(attr: TokenStream, input: TokenStream) -> TokenStream {
    let entries = match Punctuated::<Expr, Comma>::parse_terminated.parse(attr) {
        Ok(e) => e,
        Err(err) => return err.to_compile_error().into(),
    };
    let input = syn::parse_macro_input!(input as ItemFn);

    new_multi_handler(entries, input)
}
