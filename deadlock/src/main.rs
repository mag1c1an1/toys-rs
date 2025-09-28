use std::time::Duration;

use tokio::runtime;

async fn async_func() {
    tokio::time::sleep(Duration::from_micros(1000)).await;
    println!("after async sleep")
}

fn sync_func() {
    futures::executor::block_on(async { async_func().await });
}

fn main() {
    let rt = runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .unwrap();
    rt.block_on(async {
        sync_func();
    })
}
