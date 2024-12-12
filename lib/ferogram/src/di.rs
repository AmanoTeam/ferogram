// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Dependency injection module.

use futures_util::Future;
use std::{
    any::{Any, TypeId},
    collections::{hash_map::Entry, HashMap, VecDeque},
    marker::PhantomData,
    sync::Arc,
};

use async_trait::async_trait;

use crate::Result;

/// Endpoint type.
///
/// A boxed [`Handler`].
pub type Endpoint = Box<dyn Handler>;

/// Dependency injector.
///
/// Used to inject dependencies into handlers.
#[derive(Clone, Debug, Default)]
pub struct Injector {
    resources: HashMap<TypeId, VecDeque<Resource>>,
}

impl Injector {
    /// Count of resources stored.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let injector = unimplemented!();
    /// let count = injector.len();
    /// # }
    /// ```
    pub fn len(&self) -> usize {
        self.resources.len()
    }

    /// Insert a new resource.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let injector = unimplemented!();
    /// injector.insert(String::from("Hello, world!"));
    /// # }
    /// ```
    pub fn insert<R: Clone + Send + Sync + 'static>(&mut self, value: R) {
        self.resources
            .entry(TypeId::of::<R>())
            .or_insert_with(VecDeque::new)
            .push_back(Resource::new(value));
    }

    /// Insert a new resource.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let injector = unimplemented!();
    /// let injector = injector.with(String::from("Hello, world!"));
    /// # }
    /// ```
    pub fn with<R: Clone + Send + Sync + 'static>(mut self, value: R) -> Self {
        self.insert(value);
        self
    }

    /// Extend the resources with the resources of another injector.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let injector = unimplemented!();
    /// let injector = injector.extend(&mut Injector::default());
    /// # }
    /// ```
    pub fn extend(&mut self, other: &mut Self) {
        for (type_id, values) in other.resources.drain() {
            self.resources
                .entry(type_id)
                .or_insert_with(VecDeque::new)
                .extend(values);
        }
    }

    /// Remove a resource.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let injector = unimplemented!();
    /// let resource = injector.take::<String>();
    /// # }
    /// ```
    pub fn take<R: Send + Sync + 'static>(&mut self) -> Option<Arc<R>> {
        match self.resources.entry(TypeId::of::<R>()) {
            Entry::Occupied(mut e) => e.get_mut().pop_front().unwrap().to(),
            Entry::Vacant(_) => None,
        }
    }

    /// Get a reference for a resource.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let injector = unimplemented!();
    /// let resource = injector.get::<String>();
    /// # }
    /// ```
    pub fn get<R: Send + Sync + 'static>(&mut self) -> Option<&R> {
        self.resources
            .get(&TypeId::of::<R>())
            .and_then(|values| values.front())
            .and_then(|resource| resource.to_ref())
    }
}

/// A resource.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Resource {
    type_name: &'static str,
    value: Value,
}

impl Resource {
    /// Create a new injectable resource.
    pub fn new<T: Send + Sync + 'static>(value: T) -> Self {
        Self {
            type_name: std::any::type_name::<T>(),
            value: Arc::new(value),
        }
    }

    /// Downcast the resource.
    pub fn to<T: Send + Sync + 'static>(self) -> Option<Arc<T>> {
        self.value.downcast().ok()
    }

    /// Downcast the resource to a reference.
    pub fn to_ref<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.value.downcast_ref()
    }
}

/// A resource value.
pub type Value = Arc<dyn Any + Send + Sync>;

#[async_trait]
/// Handler trait, used to handle the request.
pub trait Handler: CloneHandler + Send + Sync + 'static {
    /// Handles the request.
    async fn handle(&mut self, injector: &mut Injector) -> Result<()>;
}

macro_rules! impl_handler {
    ($($params:ident),*) => {
        #[async_trait]
        impl<Fut: ?Sized, Output, $($params),*> Handler for HandlerFunc<($($params,)*), Fut>
        where
            Fut: FnMut($($params),*) -> Output + Clone + Send + Sync + 'static,
            Output: Future<Output = Result<()>> + Send + Sync + 'static,
            $($params: Clone + Send + Sync + 'static,)*
        {
            #[inline]
            #[allow(unused_mut)]
            #[allow(non_snake_case)]
            #[allow(unused_variables)]
            async fn handle(&mut self, injector: &mut Injector) -> Result<()> {
                $(
                    let $params = std::borrow::Borrow::<$params>::borrow(match injector.take() {
                        Some(ref value) => value,
                        None => return Err(format!("Missing dependency: {:?}", stringify!($params)).into()),
                    })
                    .clone();
                )*

                (self.f)($($params),*).await
            }
        }
    };
}

impl_handler!();
impl_handler!(A);
impl_handler!(A, B);
impl_handler!(A, B, C);
impl_handler!(A, B, C, D);
impl_handler!(A, B, C, D, E);
impl_handler!(A, B, C, D, E, F);
impl_handler!(A, B, C, D, E, F, G);
impl_handler!(A, B, C, D, E, F, G, H);
impl_handler!(A, B, C, D, E, F, G, H, I);
impl_handler!(A, B, C, D, E, F, G, H, I, J);
impl_handler!(A, B, C, D, E, F, G, H, I, J, K);
impl_handler!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_handler!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_handler!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_handler!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_handler!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
impl_handler!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);

/// Handler function holder.
#[derive(Clone)]
pub struct HandlerFunc<Input, F> {
    /// The function.
    f: F,
    /// The marker.
    marker: PhantomData<fn() -> Input>,
}

/// A trait that allows converting a function into a [`Handler`].
pub trait IntoHandler<Input>: Send {
    type Handler: Handler + Send;

    /// Converts the function into a [`Handler`].
    fn into_handler(self) -> Self::Handler;
}

macro_rules! impl_into_handler {
    ($($params:ident),*) => {
        impl<Fut: ?Sized, Output, $($params),*> IntoHandler<($($params,)*)> for Fut
        where
            Fut: FnMut($($params),*) -> Output + Clone + Send + Sync + 'static,
            Output: Future<Output = Result<()>> + Send + Sync + 'static,
            $($params: Clone + Send + Sync + 'static,)*
        {
            type Handler = HandlerFunc<($($params,)*), Self>;

            fn into_handler(self) -> Self::Handler {
                HandlerFunc {
                    f: self,
                    marker: Default::default(),
                }
            }
        }
    };
}

impl_into_handler!();
impl_into_handler!(A);
impl_into_handler!(A, B);
impl_into_handler!(A, B, C);
impl_into_handler!(A, B, C, D);
impl_into_handler!(A, B, C, D, E);
impl_into_handler!(A, B, C, D, E, F);
impl_into_handler!(A, B, C, D, E, F, G);
impl_into_handler!(A, B, C, D, E, F, G, H);
impl_into_handler!(A, B, C, D, E, F, G, H, I);
impl_into_handler!(A, B, C, D, E, F, G, H, I, J);
impl_into_handler!(A, B, C, D, E, F, G, H, I, J, K);
impl_into_handler!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_into_handler!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_into_handler!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_into_handler!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_into_handler!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
impl_into_handler!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);

/// A trait that allows cloning the handler.
pub trait CloneHandler {
    /// Clones the handler.
    fn clone_handler(&self) -> Box<dyn Handler>;
}

impl<T: Handler + Clone> CloneHandler for T {
    fn clone_handler(&self) -> Box<dyn Handler> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Handler> {
    fn clone(&self) -> Self {
        self.clone_handler()
    }
}
