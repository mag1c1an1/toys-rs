/*
* const col = 30
   // Clear the screen by printing \x0c.
   bar := fmt.Sprintf("\x0c[%%-%vs]", col)
   for i := 0; i < col; i++ {
       fmt.Printf(bar, strings.Repeat("=", i)+">")
       time.Sleep(100 * time.Millisecond)
   }
   fmt.Printf(bar+" Done!", strings.Repeat("=", col))
   */

use core::time;
use std::thread;

fn main() {
    let col = 30;
    for i in 0..col {
        println!("\x0c[{}]", "=".repeat(i) + ">");
        thread::sleep(time::Duration::from_millis(100));
    }
    println!("\x0c[{}] Done!", "=".repeat(col));
}
