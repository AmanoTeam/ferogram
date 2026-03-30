// Copyright 2024-2026 - Andriel Ferreira
//
// Licensed under the MIT license <LICENSE or https://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

//! Dependency injector for [`crate::Handler`]'s endpoint.

use std::{
    any::{Any, TypeId},
    borrow::Borrow,
    collections::VecDeque,
    marker::PhantomData,
    sync::Arc,
};

use dashmap::{DashMap, Entry};
use futures_core::future::BoxFuture;

use crate::error::InjectorError;

type Value = Arc<dyn Any + Send + Sync>;

/// A boxed [`RequestHandler`].
pub(crate) type Endpoint = Box<dyn RequestHandler>;

/// Dependency injector.
#[derive(Clone, Debug, Default)]
pub struct Injector {
    resources: DashMap<TypeId, VecDeque<Resource>>,
}

impl Injector {
    /// How many resources are stored.
    pub fn len(&self) -> usize {
        self.resources.len()
    }

    /// Check if the resource list is empty.
    pub fn is_empty(&self) -> bool {
        self.resources.is_empty()
    }

    /// Push a new resource.
    pub fn push<R: Clone + Send + Sync + 'static>(&mut self, value: R) {
        self.resources
            .entry(TypeId::of::<R>())
            .or_default()
            .push_back(Resource::new(value));
    }

    /// Remove a resource.
    pub fn take<R: Send + Sync + 'static>(&mut self) -> Option<Arc<R>> {
        match self.resources.entry(TypeId::of::<R>()) {
            Entry::Occupied(mut e) => e.get_mut().pop_front().unwrap().to(),
            Entry::Vacant(_) => None,
        }
    }

    /// Extend the resource list extracting the resource list of another injector.
    pub fn extend(&mut self, other: Self) {
        self.resources.extend(other.resources);
    }

    /// Update a resource through a closure.
    pub fn update<R: Clone + Send + Sync + 'static>(
        &mut self,
        f: impl FnOnce(R) -> R,
    ) -> Result<(), InjectorError> {
        match self.resources.entry(TypeId::of::<R>()) {
            Entry::Occupied(mut e) => {
                let resource = e
                    .get_mut()
                    .pop_front()
                    .unwrap()
                    .to::<R>()
                    .expect("Failed to downcast");
                let resource = f(Borrow::<R>::borrow(&resource).clone());
                let resource = Resource::new(resource);
                e.get_mut().push_front(resource);

                Ok(())
            }
            Entry::Vacant(_) => Err(InjectorError::MissingDependency(
                std::any::type_name::<R>().to_string(),
            )),
        }
    }
}

/// A injectable resource.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Resource {
    type_name: &'static str,
    value: Value,
}

impl Resource {
    pub fn new<T: Send + Sync + 'static>(value: T) -> Self {
        Self {
            type_name: std::any::type_name::<T>(),
            value: Arc::new(value),
        }
    }

    /// Downcast the resource to an owned.
    pub fn to<T: Send + Sync + 'static>(self) -> Option<Arc<T>> {
        self.value.downcast().ok()
    }

    /// Downcast the resource to a reference.
    pub fn to_ref<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.value.downcast_ref()
    }
}

pub trait RequestHandler: Send + Sync + 'static {
    /// Handle the request.
    fn handle(&self, injector: Injector) -> BoxFuture<'_, crate::handler::Result>;
}

macro_rules! impl_request_handler {
    ($($param:ident),*) => {
        impl<Fut, Output, $($param),*> RequestHandler for RequestHandlerFunc<($($param,)*), Fut>
        where
            Fut: Fn($($param),*) -> Output + Send + Sync + 'static,
            Output: Future<Output = crate::handler::Result> + Send,
            $($param: Clone + Send + Sync + 'static,)*
        {
            #[inline]
            #[allow(unused_mut)]
            #[allow(non_snake_case)]
            #[allow(unused_variables)]
            fn handle(&self, mut injector: Injector) -> BoxFuture<'_, crate::handler::Result> {
                $(
                    let $param = Borrow::<$param>::borrow(match injector.take() {
                        Some(ref value) => value,
                        None => return Box::pin(async move { Err(crate::error::InjectorError::MissingDependency(stringify!($param).to_string()).into()) }),
                    })
                    .clone();
                )*

                Box::pin(async move { (self.f)($($param),*).await })
            }
        }
    }
}

impl_request_handler!();
impl_request_handler!(A);
impl_request_handler!(A, B);
impl_request_handler!(A, B, C);
impl_request_handler!(A, B, C, D);
impl_request_handler!(A, B, C, D, E);
impl_request_handler!(A, B, C, D, E, F);
impl_request_handler!(A, B, C, D, E, F, G);
impl_request_handler!(A, B, C, D, E, F, G, H);
impl_request_handler!(A, B, C, D, E, F, G, H, I);
impl_request_handler!(A, B, C, D, E, F, G, H, I, J);
impl_request_handler!(A, B, C, D, E, F, G, H, I, J, K);
impl_request_handler!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_request_handler!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_request_handler!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_request_handler!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_request_handler!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
impl_request_handler!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);

/// Request handler function holder.
pub struct RequestHandlerFunc<Input, F> {
    f: F,
    marker: PhantomData<fn() -> Input>,
}

/// Allow converting a function into a [`RequestHandler`].
pub trait IntoRequestHandler<Input>: Send {
    type Handler: RequestHandler + Send;

    /// Convert the function into [`RequestHandler`].
    fn into_handler(self) -> Self::Handler;
}

macro_rules! impl_into_request_handler {
    ($($param:ident),*) => {
        impl<Fut, Output, $($param),*> IntoRequestHandler<($($param,)*)> for Fut
        where
            Fut: Fn($($param),*) -> Output + Send + Sync + 'static,
            Output: Future<Output = crate::Result<()>> + Send,
            $($param: Clone + Send + Sync + 'static,)*
        {
            type Handler = RequestHandlerFunc<($($param,)*), Self>;

            fn into_handler(self) -> Self::Handler {
                RequestHandlerFunc {
                    f: self,
                    marker: Default::default(),
                }
            }
        }
    };
}

impl_into_request_handler!();
impl_into_request_handler!(A);
impl_into_request_handler!(A, B);
impl_into_request_handler!(A, B, C);
impl_into_request_handler!(A, B, C, D);
impl_into_request_handler!(A, B, C, D, E);
impl_into_request_handler!(A, B, C, D, E, F);
impl_into_request_handler!(A, B, C, D, E, F, G);
impl_into_request_handler!(A, B, C, D, E, F, G, H);
impl_into_request_handler!(A, B, C, D, E, F, G, H, I);
impl_into_request_handler!(A, B, C, D, E, F, G, H, I, J);
impl_into_request_handler!(A, B, C, D, E, F, G, H, I, J, K);
impl_into_request_handler!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_into_request_handler!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_into_request_handler!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_into_request_handler!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_into_request_handler!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
impl_into_request_handler!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);
