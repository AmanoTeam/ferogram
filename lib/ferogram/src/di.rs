// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use futures::Future;
use std::{
    any::{Any, TypeId},
    collections::{hash_map::Entry, HashMap},
    marker::PhantomData,
    sync::Arc,
};

use async_trait::async_trait;

use crate::Result;

/// Endpoint type.
///
/// A boxed `Handler`.
pub type Endpoint = Box<dyn Handler>;

/// Dependency injector.
///
/// Used to inject dependencies into handlers.
#[derive(Clone, Default)]
pub struct Injector {
    resources: HashMap<TypeId, Vec<Resource>>,
}

impl Injector {
    /// Count of resources stored.
    pub fn len(&self) -> usize {
        self.resources.len()
    }

    /// Insert a new resource.
    pub fn insert<R: Send + Sync + 'static>(&mut self, value: R) {
        self.resources
            .entry(TypeId::of::<R>())
            .or_insert_with(Vec::new)
            .push(Resource::new(value));
    }

    /// Extend the resources with the resources of another injector.
    pub fn extend(&mut self, other: &mut Self) {
        for (type_id, values) in other.resources.drain() {
            self.resources
                .entry(type_id)
                .or_insert_with(Vec::new)
                .extend(values);
        }
    }

    /// Remove a resource.
    pub fn take<R: Send + Sync + 'static>(&mut self) -> Option<Arc<R>> {
        match self.resources.entry(TypeId::of::<R>()) {
            Entry::Occupied(mut e) => e.get_mut().pop().unwrap().value.downcast().ok(),
            Entry::Vacant(_) => None,
        }
    }
}

/// A resource.
#[allow(dead_code)]
#[derive(Clone)]
pub struct Resource {
    type_name: &'static str,
    value: Arc<dyn Any + Send + Sync>,
}

impl Resource {
    /// Create a new injectable resource.
    pub fn new<T: Send + Sync + 'static>(value: T) -> Self {
        Self {
            type_name: std::any::type_name::<T>(),
            value: Arc::new(value),
        }
    }
}

#[async_trait]
/// Handler trait.
pub trait Handler: CloneHandler + Send + Sync + 'static {
    async fn handle(&mut self, injector: &mut Injector) -> Result<()>;
}

macro_rules! impl_handler {
    ($($params:ident),*) => {
        #[async_trait]
        impl<Fut, Output, $($params),*> Handler for HandlerFunc<($($params,)*), Fut>
        where
            Fut: FnMut($($params),*) -> Output + Clone + Send + Sync + 'static,
            Output: Future<Output = Result<()>> + Send + Sync + 'static,
            $($params: Clone + Send + Sync + 'static,)*
            Self: Sized,
        {
            #[inline]
            #[allow(unused_mut)]
            #[allow(non_snake_case)]
            #[allow(unused_variables)]
            async fn handle(&mut self, injector: &mut Injector) -> Result<()> {
                $(
                    let $params = std::borrow::Borrow::<$params>::borrow(&match injector.take::<$params>() {
                        Some(value) => value,
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

#[derive(Clone)]
pub struct HandlerFunc<Input, F> {
    f: F,
    marker: PhantomData<fn() -> Input>,
}

/// Converts a function into a `Handler`.
pub trait IntoHandler<Input>: Send {
    type Handler: Handler + Send;

    fn into_handler(self) -> Self::Handler;
}

macro_rules! impl_into_handler {
    ($($params:ident),*) => {
        impl<Fut, Output, $($params),*> IntoHandler<($($params,)*)> for Fut
        where
            Fut: FnMut($($params),*) -> Output + Clone + Send + Sync + 'static,
            Output: Future<Output = Result<()>> + Send + Sync + 'static,
            $($params: Clone + Send + Sync + 'static ,)*
            Self: Sized,
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

/// Clone the handler trait.
pub trait CloneHandler {
    fn clone_handler(&self) -> Box<dyn Handler>;
}

impl<T> CloneHandler for T
where
    T: Handler + Clone,
{
    fn clone_handler(&self) -> Box<dyn Handler> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Handler> {
    fn clone(&self) -> Self {
        self.clone_handler()
    }
}
