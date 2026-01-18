
use crate::RookLWResult;
use crate::prodcon::OnProduceCallback;
use super::ProducerCallbacks;

use crossbeam_channel::Sender;

pub trait ProducerTask<T: Send + Clone + 'static>: Send {

    fn get_producer_callbacks(&mut self) -> &mut ProducerCallbacks<T>;

    fn on_produce<F>(&mut self, callback: F)
        where F: OnProduceCallback<T>
    {
        self.get_producer_callbacks().on_produce(callback);
    }

    fn connect(&mut self, sender: Sender<T>) {
        self.on_produce(move |item| {
            Ok(sender.send(item.clone())?)
        });
    }

    fn produce(&mut self, item: T) -> RookLWResult<()> {
        self.get_producer_callbacks().produce(&item)
    }
}