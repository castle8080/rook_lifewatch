mod consumer_task;
mod producer_callbacks;
mod producer_task;

pub use consumer_task::*;
pub use producer_callbacks::*;
pub use producer_task::*;

use crate::RookLWResult;

pub trait OnProduceCallback<T> : Fn(&T) -> RookLWResult<()> + Send + 'static {}

impl<T, F> OnProduceCallback<T> for F where F: Fn(&T) -> RookLWResult<()> + Send + 'static {}

pub trait ProducerConsumerTask<T: Send + Clone + 'static>: ProducerTask<T> + ConsumerTask<T> {}