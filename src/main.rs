extern crate rftables;

use rftables::*;

fn main() {
    let chain = Chain::load(Family::Inet, "filter", "pote").unwrap();
    println!("{:?}", chain);
}
