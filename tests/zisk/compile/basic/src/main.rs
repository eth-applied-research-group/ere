#![no_main]

ziskos::entrypoint!(main);

fn main() {
    // Read an input
    let n = u32::from_le_bytes(
        ziskos::read_input()
            .try_into()
            .expect("input to be 4 bytes"),
    );
    // Write n*2 to output
    ziskos::set_output(0, n * 2);
}
