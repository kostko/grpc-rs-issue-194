extern crate grpcio;
extern crate futures;

use std::sync::Arc;

use futures::sync::{mpsc, oneshot};
use futures::{future, Future, Stream};

fn main() {
    let environment = Arc::new(grpcio::Environment::new(4));

    // Create an executor based on grpcio. This hack is required because the
    // spawn functionality is not otherwise exposed.
    let executor = grpcio::Client::new(grpcio::ChannelBuilder::new(environment).connect(""));

    let (ch1_tx, ch1_rx) = mpsc::unbounded::<oneshot::Sender<()>>();
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    let future1 = ch1_rx.for_each(|item| {
        item.send(())
    }).boxed();
    let future2 = future::lazy(move || {
        let (tx, rx) = oneshot::channel::<()>();
        drop(ch1_tx.unbounded_send(tx));

        rx.map_err(|_| ()).and_then(move |_| shutdown_tx.send(()))
    }).boxed();

    let joined = future::join_all(vec![future1, future2]);

    executor.spawn(joined.map(|_| ()));

    shutdown_rx.wait().unwrap();
}
