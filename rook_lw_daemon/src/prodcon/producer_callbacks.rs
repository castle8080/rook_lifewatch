use crate::RookLWResult;
use super::OnProduceCallback;

pub struct ProducerCallbacks<T: Send + 'static> {
    callbacks: Vec<Box<dyn OnProduceCallback<T>>>,
}

impl<T: Send + 'static> ProducerCallbacks<T> {
    pub fn new() -> Self {
        Self {
            callbacks: Vec::new(),
        }
    }

    pub fn on_produce<F>(&mut self, callback: F)
        where F: OnProduceCallback<T>
    {
        self.callbacks.push(Box::new(callback));
    }

    pub fn produce(&self, item: &T) -> RookLWResult<()> {
        for callback in &self.callbacks {
            callback(item)?;
        }
        Ok(())
    }
}