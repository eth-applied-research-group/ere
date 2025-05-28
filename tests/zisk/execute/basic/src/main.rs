#![no_main]

ziskos::entrypoint!(main);

fn main() {
    let input = ziskos::read_input();
    if input.len() != 6 {
        std::process::exit(1);
    }

    // Read an input
    let n = u32::from_le_bytes(input[..4].try_into().unwrap());
    let a = u16::from_le_bytes(input[4..6].try_into().unwrap()) as u32;

    ziskos::set_output(0, (n + a) * 2);
}
