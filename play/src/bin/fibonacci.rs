fn fib() -> impl FnMut() -> i32 {
    let (mut a, mut b) = (0, 1);
    move || {
        (a, b) = (b, a + b);
        a
    }
}

fn main() {
    let mut f = fib();
    println!("{} {} {} {} {}", f(), f(), f(), f(), f())
}
