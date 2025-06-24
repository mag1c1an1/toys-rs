// SPDX-FileCopyrightText: LakeSoul Contributors
//
// SPDX-License-Identifier: Apache-2.0

use std::pin::Pin;

use arrow::{array::RecordBatch, datatypes::SchemaRef};
use futures::Stream;

pub trait RecordBatchStream: Stream<Item = RecordBatch> {
    fn schema(&self) -> SchemaRef;
}

pub type SendableRecordBatchStream = Pin<Box<dyn RecordBatchStream + Send>>;

// pub struct ReceiverStreamBuilder<O> {
//     tx: Sender<O>,
//     rx: Receiver<O>,
//     join_set: JoinSet<()>,
// }

// impl<O: Send + 'static> ReceiverStreamBuilder<O> {
//     /// Create new channels with the specified buffer size
//     pub fn new(capacity: usize) -> Self {
//         let (tx, rx) = tokio::sync::mpsc::channel(capacity);

//         Self {
//             tx,
//             rx,
//             join_set: JoinSet::new(),
//         }
//     }

//     /// Get a handle for sending data to the output
//     pub fn tx(&self) -> Sender<O> {
//         self.tx.clone()
//     }

//     /// Spawn task that will be aborted if this builder (or the stream
//     /// built from it) are dropped
//     pub fn spawn<F>(&mut self, task: F)
//     where
//         F: Future<Output = ()>,
//         F: Send + 'static,
//     {
//         self.join_set.spawn(task);
//     }

//     /// Spawn a blocking task that will be aborted if this builder (or the stream
//     /// built from it) are dropped.
//     ///
//     /// This is often used to spawn tasks that write to the sender
//     /// retrieved from `Self::tx`.
//     pub fn spawn_blocking<F>(&mut self, f: F)
//     where
//         F: FnOnce() -> (),
//         F: Send + 'static,
//     {
//         self.join_set.spawn_blocking(f);
//     }

//     /// Create a stream of all data written to `tx`
//     pub fn build(self) -> BoxStream<'static, O> {
//         let Self {
//             tx,
//             rx,
//             mut join_set,
//         } = self;

//         // Doesn't need tx
//         drop(tx);

//         // future that checks the result of the join set, and propagates panic if seen
//         let check = async move {
//             let _output = join_set.join_all().await;
//         };

//         let check_stream = futures::stream::once(check);
//         // unwrap Option / only return the error

//         // Convert the receiver into a stream
//         let rx_stream = futures::stream::unfold(rx, |mut rx| async move {
//             let next_item = rx.recv().await;
//             next_item.map(|next_item| (next_item, rx))
//         });

//         // Merge the streams together so whichever is ready first
//         // produces the batch
//         futures::stream::select(rx_stream, check_stream).boxed()
//     }
// }
