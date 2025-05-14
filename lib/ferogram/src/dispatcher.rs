// Copyright 2024-2025 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Dispatcher module.

use grammers_client::{Client, Update, types::Chat};
use tokio::sync::broadcast::Sender;

use crate::{
    Cache, Context, Plugin, Result, Router, di, filters::Command, middleware::MiddlewareStack,
};

/// A dispatcher.
///
/// Sends the updates to the routers and plugins.
#[derive(Clone)]
pub struct Dispatcher {
    /// The routers.
    routers: Vec<Router>,
    /// The plugins.
    plugins: Vec<Plugin>,
    /// The main injector.
    injector: di::Injector,
    /// The middleware stack.
    middlewares: MiddlewareStack,
    /// The update sender.
    pub(crate) upd_sender: Sender<Update>,

    /// Whether allow the client to handle updates from itself.
    allow_from_self: bool,
}

impl Dispatcher {
    /// Attachs a new router.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let dispatcher = unimplemented!();
    /// let dispatcher = dispatcher.router(|router| {
    ///     router
    /// });
    /// # }
    /// ```
    pub fn router<R: FnOnce(Router) -> Router + 'static>(mut self, router: R) -> Self {
        let router = router(Router::default());
        self.routers.push(router);

        self
    }

    /// Attachs a injector.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let dispatcher = unimplemented!();
    /// let dispatcher = dispatcher.resources(|injector| {
    ///     injector.insert(String::from("Hello, world!"));
    /// });
    /// # }
    /// ```
    pub fn resources<D: FnOnce(di::Injector) -> di::Injector>(mut self, injector: D) -> Self {
        let mut injector = injector(di::Injector::default());
        self.injector.extend(&mut injector);

        self
    }

    /// Attachs a injector.
    ///
    /// Same as `resources`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let dispatcher = unimplemented!();
    /// let dispatcher = dispatcher.dependencies(|injector| {
    ///     injector.insert(String::from("Hello, world!"));
    /// });
    /// # }
    /// ```
    pub fn dependencies<D: FnOnce(di::Injector) -> di::Injector>(self, injector: D) -> Self {
        self.resources(injector)
    }

    /// Attachs a middleware stack.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let dispatcher = unimplemented!();
    /// let dispatcher = dispatcher.middlewares(|middlewares| {
    ///     middlewares
    ///         .before(|_, _, _| async { Ok(flow::continue_now()) })
    ///         .after(|_, _, _| async { Ok(flow::continue_now()) })
    /// });
    /// # }
    /// ```
    pub fn middlewares<M: FnOnce(MiddlewareStack) -> MiddlewareStack>(
        mut self,
        middlewares: M,
    ) -> Self {
        self.middlewares = middlewares(self.middlewares);
        self
    }

    /// Allows the client to handle updates from itself.
    ///
    /// By default, the client will not handle updates from itself.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let dispatcher = unimplemented!();
    /// let dispatcher = dispatcher.allow_from_self();
    /// # }
    /// ```
    pub fn allow_from_self(mut self) -> Self {
        self.allow_from_self = true;
        self
    }

    /// Attachs a new plugin.
    ///
    /// A plugin is a collection of routers.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let dispatcher = unimplemented!();
    /// let dispatcher = dispatcher.plugin(Plugin::default());
    /// # }
    /// ```
    pub fn plugin(mut self, plugin: Plugin) -> Self {
        self.plugins.push(plugin);
        self
    }

    /// Returns the commands from the routers and plugins.
    pub(crate) fn get_commands(&self) -> Vec<Command> {
        let mut commands = Vec::new();

        commands.extend(self.routers.iter().flat_map(|router| router.get_commands()));
        commands.extend(
            self.plugins
                .iter()
                .flat_map(|plugin| plugin.router.get_commands()),
        );

        commands
    }

    /// Handle the update sent by Telegram.
    ///
    /// Returns `Ok(())` if the update was handled.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let dispatcher = unimplemented!();
    /// let dispatcher = dispatcher.handle_update(&client, &update).await?;
    /// # }
    /// ```
    pub(crate) async fn handle_update(
        &mut self,
        cache: &Cache,
        client: &Client,
        update: &Update,
    ) -> Result<()> {
        let mut injector = di::Injector::default();

        let upd_receiver = self.upd_sender.subscribe();
        let context = Context::with(cache, client, update, upd_receiver);
        injector.insert(context);

        self.upd_sender
            .send(update.clone())
            .expect("Failed to send update");

        injector.insert(client.clone());
        injector.insert(update.clone());
        injector.extend(&mut self.injector.clone());

        match update {
            Update::NewMessage(message) | Update::MessageEdited(message) => {
                let chat = message.chat();
                cache.save_chat(chat.pack()).await?;

                if let Some(Chat::User(user)) = message.sender() {
                    cache.save_chat(user.pack()).await?;

                    if !self.allow_from_self && user.is_self() {
                        return Ok(());
                    }
                }
            }
            Update::CallbackQuery(query) => {
                if let Chat::User(user) = query.sender() {
                    cache.save_chat(user.pack()).await?;

                    if !self.allow_from_self && user.is_self() {
                        return Ok(());
                    }
                }
            }
            Update::InlineQuery(query) => {
                let user = query.sender();
                cache.save_chat(user.pack()).await?;

                if !self.allow_from_self && user.is_self() {
                    return Ok(());
                }
            }
            Update::InlineSend(inline_send) => {
                let user = inline_send.sender();
                cache.save_chat(user.pack()).await?;

                if user.is_self() {
                    return Ok(());
                }
            }
            _ => {}
        };

        for router in self.routers.iter_mut() {
            match router
                .handle_update(client, update, &mut injector, self.middlewares.clone())
                .await
            {
                Ok(false) => continue,
                Ok(true) => return Ok(()),
                Err(e) => return Err(e),
            }
        }

        for plugin in self.plugins.iter_mut() {
            match plugin
                .router
                .handle_update(client, update, &mut injector, self.middlewares.clone())
                .await
            {
                Ok(false) => continue,
                Ok(true) => return Ok(()),
                Err(e) => return Err(e),
            }
        }

        Ok(())
    }
}

impl Default for Dispatcher {
    fn default() -> Self {
        let (upd_sender, _) = tokio::sync::broadcast::channel(10);

        Self {
            routers: Vec::new(),
            plugins: Vec::new(),
            injector: di::Injector::default(),
            middlewares: MiddlewareStack::new(),
            upd_sender,

            allow_from_self: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handler;

    #[test]
    fn test_dispatcher() {
        Dispatcher::default()
            .router(|router| router)
            .router(|router| {
                router.register(handler::then(|_: Client, _: Update| async { Ok(()) }))
            });
    }
}
