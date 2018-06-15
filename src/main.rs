#![allow(deprecated)]

extern crate grpcio;
extern crate futures;
#[cfg(feature = "cpupool")]
extern crate futures_cpupool;

#[cfg(not(feature = "cpupool"))]
use std::sync::Arc;

use futures::sync::{mpsc, oneshot};
use futures::{future, Future, Stream};

fn main() {
    // Create an executor based on grpcio. This hack is required because the
    // spawn functionality is not otherwise exposed.
    #[cfg(not(feature = "cpupool"))]
    let executor = {
        let environment = Arc::new(grpcio::Environment::new(4));
        grpcio::Client::new(grpcio::ChannelBuilder::new(environment).connect(""))
    };
    #[cfg(feature = "cpupool")]
    let executor = futures_cpupool::CpuPool::new(4);

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

    drop(executor.spawn(joined.map(|_| ())));

    drop(shutdown_rx.wait());
}
