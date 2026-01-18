use crate::RookLWResult;

use std::thread::{JoinHandle, spawn};
use std::any::type_name;

use tracing::info;
use crossbeam_channel::Receiver;

pub trait ConsumerTask<T: Send + 'static>: Send {
    fn consume(&mut self, item: T) -> RookLWResult<()>;

    fn run_listener(&mut self, item: Receiver<T>) -> RookLWResult<()> {
        for received in item.iter() {
            self.consume(received)?;
        }
        Ok(())
    }

    fn start_listener(self, receiver: Receiver<T>) -> JoinHandle<RookLWResult<()>>
    where Self: Sized + 'static
    {
        self.spawn_listener(receiver)

    }

    fn start_listener_boxed(self: Box<Self>, receiver: Receiver<T>) -> JoinHandle<RookLWResult<()>>
    where Self: Sized + 'static
    {
        self.spawn_listener(receiver)
    }

    fn spawn_listener(mut self, receiver: Receiver<T>) -> JoinHandle<RookLWResult<()>>
    where Self: Sized + 'static
    {
        spawn(move || {
            info!(
                consumer_type = %type_name::<Self>(),
                "Starting consumer listener"
            );
            match self.run_listener(receiver) {
                Ok(_) => {
                    info!(
                        consumer_type = %type_name::<Self>(),
                        "Consumer listener exiting normally"
                    );
                    Ok(())
                },
                Err(e) => {
                    info!(
                        consumer_type = %type_name::<Self>(),
                        error = %e,
                        "Consumer listener exiting with error"
                    );
                    Err(e)
                }
            }
        })
    }
}