#[doc(inline)]
pub use std; // for documentation purposes
use std::io::{self, Write};
use components::CanvasParts;
use crossterm::{cursor::MoveTo, execute, queue, style::{Color, PrintStyledContent, Stylize}, terminal::{enable_raw_mode, EnableLineWrap}, QueueableCommand};
use lazy_static::lazy_static; 
//lazy_static is ok, mutability not needed
use rand::{thread_rng, Rng};
use std::time::Duration;
use std::thread::sleep;
use std::time::Instant;
use std::collections::HashMap;

mod part_handler;
mod components;

lazy_static! {
    static ref TERM_SIZE: (u16, u16) = crossterm::terminal::size().unwrap_or_else(|_| panic!("Cannot get terminal size"));
}

lazy_static!{
    static ref ELEMENT_VISUALS: HashMap<components::Element, char> = HashMap::from([
        (components::Element::Wall, 'H'),
        (components::Element::Spawn, '@'),
        (components::Element::BodyPartVert, '|'),
        (components::Element::BodyPartHori, '-'),
        (components::Element::BodyPartRightLean, '\\'),
        (components::Element::BodyPartLeftLean, '/'),
        (components::Element::Food, '#'),
        (components::Element::BodyPartHead, '█'),
    ]);
    }


fn create_canvas() -> CanvasParts{
    let mut rng = thread_rng();
    let mut canvas = CanvasParts {alive: Vec::new(), environment: Vec::new(), interactable: Vec::new()}; // Create an empty vector to store the parts
    let wall_color = Color::Rgb { r: 255, g: 60, b: 70 }; // Create a new color for the walls    

    for y in 0..TERM_SIZE.1 {
        if y == 0 || y == TERM_SIZE.1 - 1 {
            for x in 0..TERM_SIZE.0 {
                canvas.add_element(components::Element::Wall, (x, y), Some(wall_color), None);
            }
        } else {
            canvas.add_element(components::Element::Wall, (TERM_SIZE.0 - 1, y), Some(wall_color), None); // Use array indexing instead of tuple indexing
            canvas.add_element(components::Element::Wall, (0, y), Some(wall_color), None); // Use array indexing instead of tuple indexing
        }
    }
    for _ in 0..3{
    // Use array indexing instead of tuple indexing
    canvas.add_element(components::Element::Spawn, (rng.gen_range(1..TERM_SIZE.0-1), rng.gen_range(1..TERM_SIZE.1-1)), Some(Color::Rgb { r: 10, g: 255, b: 10 }), None);
    }
    
    canvas // Return the canvas vector
}
fn draw_canvas(canvas: &CanvasParts, cleared_coords: &mut Vec<(u16, u16)>) {
    let mut stdout = io::stdout();
    
    let mut unified_elements = canvas.unify_elements();
    unified_elements.sort_by_key(|part| (part.position.1, part.position.0));
    
    for location in cleared_coords.iter() {
        queue!(stdout, MoveTo(location.0, location.1), PrintStyledContent(" ".to_string().with(Color::Reset))).unwrap();
    }

    for part in unified_elements.iter() {
        queue!(stdout, MoveTo(part.position.0, part.position.1),
        PrintStyledContent(ELEMENT_VISUALS[&part.element].to_string().with(part.color))).unwrap();
    
    }
    cleared_coords.clear();
    stdout.flush().unwrap();
}

fn main() {
    // Prepare the terminal
    //let _ = enable_raw_mode();
    execute!(io::stdout(), EnableLineWrap).unwrap();
    let mut stdout = io::stdout();
    stdout.queue(crossterm::cursor::Hide).unwrap();
    stdout.queue(crossterm::terminal::Clear(crossterm::terminal::ClearType::All)).unwrap();
    stdout.flush().unwrap();
    // Create the canvas
    let mut canvas: CanvasParts = create_canvas();
    let mut state = components::CanvasState { iterations: 0, cleared_coords: Vec::new() }; //food_rate: 0
    draw_canvas(&canvas, &mut state.cleared_coords);
    loop {
        //let mut now = Instant::now();
        part_handler::head_handle(&mut canvas);
        part_handler::spawner_handle(&mut canvas);
        part_handler::handle_killed(&mut canvas.alive, &mut state.cleared_coords);
        part_handler::spawn_food(&mut canvas);  
        //let elapsed_calc = now.elapsed();
        //now = Instant::now();
        draw_canvas(&canvas, &mut state.cleared_coords);
        //let elapsed = now.elapsed();
        //println!("Print Elapsed: {:.2?}, Calc Elapsed: {:.2?}", elapsed, elapsed_calc);
        state.iterations += 1;
        sleep(Duration::from_millis(8));
    }
    
}