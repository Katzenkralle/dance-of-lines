#[doc(inline)]
pub use std; // for documentation purposes
use std::io::{self, Write};
use crossterm::{cursor::MoveTo, queue, QueueableCommand, style::{PrintStyledContent, Stylize, Color}, terminal::enable_raw_mode};
use lazy_static::lazy_static; 
//lazy_static is ok, mutability not needed
use rand::{thread_rng, Rng};
use std::time::Duration;
use std::thread::sleep;

mod part_handler;
mod components;

lazy_static! {
    static ref TERM_SIZE: (u16, u16) = crossterm::terminal::size().unwrap_or_else(|_| panic!("Cannot get terminal size"));
}


fn create_canvas() -> Vec<components::Part>{
    let mut rng = thread_rng();
    let mut canvas: Vec<components::Part> = Vec::new(); // Create an empty vector to store the parts
    let wall_color = Color::Rgb { r: 255, g: 60, b: 70 }; // Create a new color for the walls    

    for y in 0..TERM_SIZE.1 {
        if y == 0 || y == TERM_SIZE.1 - 1 {
            for x in 0..TERM_SIZE.0 {
                let part = components::Part::new("wall", (x, y), wall_color).unwrap(); // Use array indexing instead of tuple indexing
                canvas.push(part);
            
            }
        } else {
            canvas.push(components::Part::new("wall", (0, y), wall_color).unwrap()); // Use array indexing instead of tuple indexing
            canvas.push(components::Part::new("wall", (TERM_SIZE.0 - 1, y), wall_color).unwrap()); // Use array indexing instead of tuple indexing
        }
    }
    canvas.push(components::Part::new("spawn", (rng.gen_range(1..TERM_SIZE.0-1), rng.gen_range(1..TERM_SIZE.1-1)), Color::Rgb { r: 10, g: 255, b: 10 }).unwrap()); // Use array indexing instead of tuple indexing
    canvas // Return the canvas vector
}

fn draw_canvas(canvas: &Vec<components::Part>) {
    let mut stdout = io::stdout();
    stdout.queue(crossterm::terminal::Clear(crossterm::terminal::ClearType::All)).unwrap();
    for part in canvas {
        queue!(stdout, MoveTo(part.position.0, part.position.1),
         PrintStyledContent(part.element.visual.to_string().with(part.color))).unwrap();
    }
    stdout.flush().unwrap();
}

fn main() {
    let _ = enable_raw_mode();
    let mut stdout = io::stdout();
    stdout.queue(crossterm::cursor::Hide).unwrap();
    stdout.flush().unwrap();
    let mut state: components::CanvasState = components::CanvasState::new(create_canvas());
    draw_canvas(&state.canvas);
    loop {
        part_handler::head_handle(&mut state.canvas, &mut state.colors);
        part_handler::spawner_handle(&mut state.canvas, &mut state.colors);
        part_handler::handle_killed(&mut state.canvas);
        //part_handler::spawn_food(&mut state.canvas);
        state.canvas.sort_by_key(|part| (part.position.1, part.position.0));
        draw_canvas(&state.canvas);
        state.iterations += 1;
        sleep(Duration::from_millis(32));
    }
    
}