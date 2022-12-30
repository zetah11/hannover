#![allow(unused)]

mod bytes;
mod data;
mod float;
mod math;
mod notes;
mod sampler;
mod wavetable;

fn main() {
    pretty_env_logger::init();

    let input = b"try me!";
    let mut nibbles = bytes::NibbleStream::<3>::new(input);

    for _ in 0..30 {
        println!("{:?}", nibbles.next_note());
    }
}
