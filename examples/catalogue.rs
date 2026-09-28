//! Prints the spell catalogue with generated descriptions.
//!
//! cargo run --release --example catalogue
use arpg_machine::content::SpellId;

fn main() {
    println!("{} spells\n", SpellId::count());
    for spell in SpellId::all() {
        println!(
            "{:<10} {:<20} size {} {:>5.2}s {:<10} {}",
            spell.family().name(),
            spell.name(),
            spell.size(),
            spell.cooldown(),
            spell.delivery().name(),
            spell.description()
        );
    }
}
