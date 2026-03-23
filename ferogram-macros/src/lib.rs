// Copyright 2024-2026 - Andriel Ferreira
//
// Licensed under the MIT license <LICENSE or https://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

//! General helper macros exports.

mod handler;

use proc_macro::TokenStream;
use syn::{Expr, ItemFn};

use crate::handler::{UpdateType, new_handler};

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
