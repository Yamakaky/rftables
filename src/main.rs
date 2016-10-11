extern crate rftables;

use rftables::*;

fn main() {
    let chain = Chain::load(Family::Inet, "filter", "input").unwrap();
    println!("{:?}", chain);
    println!("{:?}", Chain::load_table(Family::Inet, "filter"));
    println!("{:?}", Table::load_all());
}
