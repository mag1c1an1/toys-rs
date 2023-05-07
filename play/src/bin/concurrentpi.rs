use std::{f64::consts::PI, sync::mpsc::channel, thread};

fn main() {
    println!("               PI:{}", PI);
    println!("Nilakanhta Series:{}", pi(5000))
}

fn pi(n: i32) -> f64 {
    let mut f = 3.0;
    let (tx, rx) = channel::<f64>();
    for i in 0..n {
        let tx = tx.clone();
        thread::spawn(move || {
            let k = i as f64;
            let val = 4.0 * ((-1 as i32).pow(i as u32) as f64)
                / ((2.0 * k + 2.0) * (2.0 * k + 3.0) * (2.0 * k + 4.0));
            tx.send(val).unwrap();
        });
    }
    for _ in 0..n {
        f += rx.recv().unwrap();
    }
    f
}
