use termion::{screen, color, terminal_size};


fn main() {
    let terminal_size = terminal_size().expect("Cannot get terminal size");
    
    /*
    Equivalent to:
    let terminal_size: (u16, u16) = terminal_size().unwrap_or_else(|_| panic!("Cannot get terminal size");

    (unwrap would just panic if the result is Err, and expect would panic with the message provided.)

    Equivalent to:
    let terminal_size: (u16, u16) = match terminal_size() {
        Ok(size) => size,
        Err(_) => panic!("Cannot get erminal Size")
    };
     */
    println!("Terminal Size: {} by {} px", terminal_size.0, terminal_size.1);
}
/*
Struckt: eigenes Datentypen
enum: Aufzählungstypen, können je nach Standartwert unterschiedliche Werte annehmen
impl: Implementierung von Methoden für Struckts
trait: Schnittstelle, die von Struckts implementiert werden kann, impl für mehrere Struckts

*/