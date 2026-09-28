//! Every spell's card tagline, to review them.
//!
//! cargo run --release --example taglines
use arpg_machine::content::SpellId;

fn main() {
    for spell in SpellId::all() {
        println!("{:<22} {}", spell.name(), spell.tagline().join(" · "));
    }
}
