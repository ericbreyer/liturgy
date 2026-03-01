use rand::Rng; // make sure you have rand in [build-dependencies]

fn main() {
    // Generate a random number
    let random_number: u32 = rand::rng().random_range(0..0xFFFFFFFF);

    // Print the cargo instruction to set an environment variable
    println!("cargo:rustc-env=TEST_FPRINT={random_number}");
}
