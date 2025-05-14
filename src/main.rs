mod constants;
mod error;
mod profile;
mod startup_window;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, Auronen!");
    Ok(())
}
