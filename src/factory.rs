use std::sync::Arc;
use crate::flow::Generator;

/// Fluent API extension for naming generators
pub trait Named {
    fn named(self, name: impl Into<String>) -> Self;
}

impl<T: Generator + ?Sized> Named for Arc<T> {
    fn named(mut self, name: impl Into<String>) -> Self {
        if let Some(gen) = Arc::get_mut(&mut self) {
            gen.set_name(name.into());
        }
        self
    }
}