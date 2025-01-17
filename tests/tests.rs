extern crate ping;

use std::time::Duration;

#[test]
fn basic() {
    let addr = "127.0.0.1".parse().unwrap();
    let timeout = Duration::from_secs(1);
    let mut socket = ping::open_socket(addr, Some(timeout), Some(64), Some(3)).unwrap();
    for _ in 0..3 {
        ping::ping(&mut socket).unwrap();
    }
}
