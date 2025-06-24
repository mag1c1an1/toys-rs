// SPDX-FileCopyrightText: LakeSoul Contributors
//
// SPDX-License-Identifier: Apache-2.0

use arrow::{array::RecordBatch, datatypes::SchemaRef};
use futures::stream::BoxStream;
use futures::{Stream, StreamExt};
use pin_project_lite::pin_project;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::JoinSet,
};

use crate::make_stream;
pub trait RecordBatchStream: Stream<Item = RecordBatch> {
    fn schema(&self) -> SchemaRef;
}

pub type SendableRecordBatchStream = Pin<Box<dyn RecordBatchStream + Send>>;

pub struct ReceiverStreamBuilder<O> {
    tx: Sender<O>,
    rx: Receiver<O>,
    join_set: JoinSet<()>,
}

impl<O: Send + 'static> ReceiverStreamBuilder<O> {
    /// Create new channels with the specified buffer size
    pub fn new(capacity: usize) -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(capacity);

        Self {
            tx,
            rx,
            join_set: JoinSet::new(),
        }
    }

    /// Get a handle for sending data to the output
    pub fn tx(&self) -> Sender<O> {
        self.tx.clone()
    }

    /// Spawn task that will be aborted if this builder (or the stream
    /// built from it) are dropped
    pub fn spawn<F>(&mut self, task: F)
    where
        F: Future<Output = ()>,
        F: Send + 'static,
    {
        self.join_set.spawn(task);
    }

    /// Spawn a blocking task that will be aborted if this builder (or the stream
    /// built from it) are dropped.
    ///
    /// This is often used to spawn tasks that write to the sender
    /// retrieved from `Self::tx`.
    pub fn spawn_blocking<F>(&mut self, f: F)
    where
        F: FnOnce(),
        F: Send + 'static,
    {
        self.join_set.spawn_blocking(f);
    }

    /// Create a stream of all data written to `tx`
    pub fn build(self) -> BoxStream<'static, O> {
        let Self {
            tx,
            rx,
            mut join_set,
        } = self;

        // Doesn't need tx
        drop(tx);

        // future that checks the result of the join set, and propagates panic if seen
        let check = async move {
            while let Some(res) = join_set.join_next().await {
                match res {
                    Ok(_task_res) => {
                        continue;
                    }
                    Err(_e) => {}
                }
            }
            None
        };

        let check_stream = futures::stream::once(check).filter_map(|item| async move { item });
        // unwrap Option / only return the error

        // Convert the receiver into a stream
        let rx_stream = futures::stream::unfold(rx, |mut rx| async move {
            let next_item = rx.recv().await;
            next_item.map(|next_item| (next_item, rx))
        });

        // Merge the streams together so whichever is ready first
        // produces the batch
        futures::stream::select(rx_stream, check_stream).boxed()
    }
}
pub struct RecordBatchReceiverStreamBuilder {
    schema: SchemaRef,
    inner: ReceiverStreamBuilder<RecordBatch>,
}

impl RecordBatchReceiverStreamBuilder {
    /// Create new channels with the specified buffer size
    pub fn new(schema: SchemaRef, capacity: usize) -> Self {
        Self {
            schema,
            inner: ReceiverStreamBuilder::new(capacity),
        }
    }
    pub fn tx(&self) -> Sender<RecordBatch> {
        self.inner.tx()
    }
    pub fn run_input(&mut self, part: usize) {
        let output = self.tx();
        self.inner.spawn(async move {
            let mut stream = make_stream(8, part);

            while let Some(item) = stream.next().await {
                if output.send(item).await.is_err() {
                    panic!("eerr")
                }
            }
        });
    }

    /// Create a stream of all [`RecordBatch`] written to `tx`
    pub fn build(self) -> SendableRecordBatchStream {
        Box::pin(RecordBatchStreamAdapter::new(
            self.schema,
            self.inner.build(),
        ))
    }
}

#[doc(hidden)]
pub struct RecordBatchReceiverStream {}

impl RecordBatchReceiverStream {
    /// Create a builder with an internal buffer of capacity batches.
    pub fn builder(schema: SchemaRef, capacity: usize) -> RecordBatchReceiverStreamBuilder {
        RecordBatchReceiverStreamBuilder::new(schema, capacity)
    }
}

pin_project! {
    /// Combines a [`Stream`] with a [`SchemaRef`] implementing
    /// [`SendableRecordBatchStream`] for the combination
    ///
    /// See [`Self::new`] for an example
    pub struct RecordBatchStreamAdapter<S> {
        schema: SchemaRef,

        #[pin]
        stream: S,
    }
}

impl<S> RecordBatchStreamAdapter<S> {
    /// Creates a new [`RecordBatchStreamAdapter`] from the provided schema and stream.
    ///
    /// Note to create a [`SendableRecordBatchStream`] you pin the result
    ///
    /// # Example
    /// ```
    /// # use arrow::array::record_batch;
    /// # use datafusion_execution::SendableRecordBatchStream;
    /// # use datafusion_physical_plan::stream::RecordBatchStreamAdapter;
    /// // Create stream of Result<RecordBatch>
    /// let batch = record_batch!(
    ///   ("a", Int32, [1, 2, 3]),
    ///   ("b", Float64, [Some(4.0), None, Some(5.0)])
    /// ).expect("created batch");
    /// let schema = batch.schema();
    /// let stream = futures::stream::iter(vec![Ok(batch)]);
    /// // Convert the stream to a SendableRecordBatchStream
    /// let adapter = RecordBatchStreamAdapter::new(schema, stream);
    /// // Now you can use the adapter as a SendableRecordBatchStream
    /// let batch_stream: SendableRecordBatchStream = Box::pin(adapter);
    /// // ...
    /// ```
    pub fn new(schema: SchemaRef, stream: S) -> Self {
        Self { schema, stream }
    }
}

impl<S> Stream for RecordBatchStreamAdapter<S>
where
    S: Stream<Item = RecordBatch>,
{
    type Item = RecordBatch;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().stream.poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.stream.size_hint()
    }
}

impl<S> RecordBatchStream for RecordBatchStreamAdapter<S>
where
    S: Stream<Item = RecordBatch>,
{
    fn schema(&self) -> SchemaRef {
        Arc::clone(&self.schema)
    }
}
