#[doc(inline)]
pub use std; // for documentation purposes
use std::{char, f32::consts::E, io::{self, Chain, Write}};
use components::CanvasParts;
use crossterm::{cursor::MoveTo, event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers}, execute, queue, style::{Color, PrintStyledContent, Stylize}, terminal::{disable_raw_mode, enable_raw_mode, DisableLineWrap, EnableLineWrap}, QueueableCommand };
use lazy_static::lazy_static; 
//lazy_static is ok, mutability not needed
use rand::{thread_rng, Rng};
use std::time::Duration;
use std::thread::sleep;
use std::time::Instant;
use std::collections::HashMap;
use std::sync::RwLock; // RwLock is ok, mutability not needed 
use std::env::args;
use std::process::exit;

mod part_handler;
mod components;
mod pathfinder;


lazy_static!{static ref MAX_THREADS: RwLock<u32> = RwLock::new(0);}

lazy_static!{static ref SPAWNERS: RwLock<u16> = RwLock::new(0);}

lazy_static!{static ref RESTART: RwLock<bool> = RwLock::new(false);}

lazy_static!{static ref  SHOW_STATS:  RwLock<bool> = RwLock::new(false);}

lazy_static!{static ref MIN_DELAY: RwLock<u64> = RwLock::new(17);}

lazy_static! {
    static ref TERM_SIZE: RwLock<(u16, u16)> = RwLock::new((0, 0));
}

lazy_static!{
    static ref ELEMENT_VISUALS: HashMap<components::Element, char> = HashMap::from([
        (components::Element::Wall, '𐲕'),
        (components::Element::Spawn, '⬟'),
        (components::Element::BodyPartVert, '|'),
        (components::Element::BodyPartHori, '-'),
        (components::Element::BodyPartRightLean, '\\'),
        (components::Element::BodyPartLeftLean, '/'),
        (components::Element::Food, '#'),
        (components::Element::BodyPartHead, '█'),
        (components::Element::WespHead, '0'),
        (components::Element::WespBody, '•'),
    ]);
    }
fn help_message() {
    println!("Usage: dance_of_lines [options]
    -i: Show stats
    -s <int>: Set spawner count
    -d <int>: Set minimum delay between frames in milliseconds
    -t <int> <int>: Set terminal size in columns and rows
    -p <int>: Set maximum thread count
    -h, --help: Show this message
    ");
    exit(0);
}

fn set_runtime_constants(cl_args: std::env::Args) {
    let mut args = cl_args.skip(1);
    let mut stats = false;
    let mut min_delay = 17;
    let mut max_term_size = (0, 0);
    let mut spawners = 4;
    let mut max_threads = 0;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-i" => stats = true,
            "-s" => spawners = args.next().unwrap().parse().unwrap_or_else(|_| panic!("Invalid spawner count")),
            "-d" => min_delay = args.next().unwrap().parse().unwrap_or_else(|_| panic!("Invalid delay")),
            "-t" => max_term_size = (args.next().unwrap().parse().unwrap_or_else(|_| panic!("Invalid terminal size")), args.next().unwrap().parse().unwrap_or_else(|_| panic!("Invalid terminal size"))),
            "-p" => max_threads = args.next().unwrap().parse().unwrap_or_else(|_| panic!("Invalid thread count")),
            "-h" | "--help" => help_message(),
            _ => panic!("Invalid argument --help for help"),
        }
    }
    if max_term_size.0 > TERM_SIZE.read().unwrap().0 || max_term_size.1 > TERM_SIZE.read().unwrap().1 {
        panic!("Terminal size too large");
    } else if max_term_size == (0, 0) {
        max_term_size = crossterm::terminal::size().unwrap_or_else(|_| panic!("Cannot get terminal size"));
    }
    *MAX_THREADS.write().unwrap() = max_threads;
    *SPAWNERS.write().unwrap() = spawners;
    *SHOW_STATS.write().unwrap() = stats;
    *MIN_DELAY.write().unwrap() = min_delay;
    *TERM_SIZE.write().unwrap() = max_term_size;
}

fn exit_handler() {
    let _ = disable_raw_mode();
    let mut stdout = io::stdout();
    stdout.queue(crossterm::cursor::Show).unwrap();
    stdout.queue(DisableLineWrap).unwrap();
    stdout.flush().unwrap();
    exit(0);
}


fn handle_kb_input() {
    if poll(Duration::from_millis(0)).unwrap() {
        match read().unwrap() {
            Event::Key(KeyEvent{code: KeyCode::Char('c'), modifiers: KeyModifiers::CONTROL, kind: _, state: _}) => exit_handler(),
            Event::Key(KeyEvent{code: KeyCode::Char('s'), modifiers: _, kind: _, state: _}) => {let curent_state = *SHOW_STATS.read().unwrap(); *SHOW_STATS.write().unwrap() = !curent_state;},
            Event::Key(KeyEvent{code: KeyCode::Char('r'), modifiers: _, kind: _, state: _}) => {*RESTART.write().unwrap() = true;},
            Event::Resize(_,_) => {*RESTART.write().unwrap() = true;},
            _ => (),
        }
    }
}


fn create_canvas() -> CanvasParts{
    let mut rng = thread_rng();
    let mut canvas = CanvasParts {alive: Vec::new(), environment: Vec::new(), interactable: Vec::new()}; // Create an empty vector to store the parts
    let wall_color = Color::Rgb { r: 255, g: 60, b: 70 }; // Create a new color for the walls    

    for y in 0..TERM_SIZE.read().unwrap().1 {
        if y == 0 || y == TERM_SIZE.read().unwrap().1 - 1 {
            for x in 0..TERM_SIZE.read().unwrap().0 {
                canvas.add_element(components::Element::Wall, (x, y), Some(wall_color), None);
            }
        } else {
            canvas.add_element(components::Element::Wall, (TERM_SIZE.read().unwrap().0 - 1, y), Some(wall_color), None); // Use array indexing instead of tuple indexing
            canvas.add_element(components::Element::Wall, (0, y), Some(wall_color), None); // Use array indexing instead of tuple indexing
        }
    }
    let spawner_ranges = (TERM_SIZE.read().unwrap().0-1) / *SPAWNERS.read().unwrap() as u16;
    for i in 0..*SPAWNERS.read().unwrap(){
    // Use array indexing instead of tuple indexing
    canvas.add_element(components::Element::Spawn, (rng.gen_range((spawner_ranges*i)..spawner_ranges*(i+1)), rng.gen_range(1..TERM_SIZE.read().unwrap().1-1)), Some(Color::Rgb { r: 10, g: 255, b: 10 }), None);
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
        if *SHOW_STATS.read().unwrap() && part.position.1 == TERM_SIZE.read().unwrap().1 - 1 {
            continue;
        } 
        queue!(stdout, MoveTo(part.position.0, part.position.1),
        PrintStyledContent(ELEMENT_VISUALS[&part.element].to_string().with(part.color))).unwrap();
    
    }
    cleared_coords.clear();
    stdout.flush().unwrap();
}

fn main() {
    set_runtime_constants(args());
    loop {
        if !args().any(|arg| arg == "-t"){
            *TERM_SIZE.write().unwrap() = crossterm::terminal::size().unwrap_or_else(|_| panic!("Cannot get terminal size"));
        }
        // Prepare the terminal
        let _ = enable_raw_mode();
        execute!(io::stdout(), EnableLineWrap).unwrap();
        let mut stdout = io::stdout();
        stdout.queue(crossterm::cursor::Hide).unwrap();
        stdout.queue(crossterm::terminal::Clear(crossterm::terminal::ClearType::All)).unwrap();
        stdout.flush().unwrap();

        // Create the canvas
        let mut canvas: CanvasParts = create_canvas();
        let mut state = components::CanvasState { iterations: 0, cleared_coords: Vec::new() }; //food_rate: 0
        draw_canvas(&canvas, &mut state.cleared_coords);
        let mut last_refresh = Instant::now();

        *RESTART.write().unwrap() = false;
        
        while !*RESTART.read().unwrap() {
            let now = Instant::now();
            pathfinder::head_handle(&mut canvas);
            part_handler::spawner_handle(&mut canvas);
            part_handler::handle_killed(&mut canvas.alive, &mut state.cleared_coords);
            part_handler::spawn_food(&mut canvas);  

            draw_canvas(&canvas, &mut state.cleared_coords);
            state.iterations += 1;

            handle_kb_input();

            let elapsed = now.elapsed();
            if elapsed < Duration::from_millis(*MIN_DELAY.read().unwrap()) {
                sleep(Duration::from_millis(*MIN_DELAY.read().unwrap()) - elapsed);
            }
            if *SHOW_STATS.read().unwrap() && last_refresh.elapsed() > Duration::from_secs_f64(0.5) {
            let mut stats_string = format!("Iterations:{}|FPS:{:.2?}|Creatures:{}|<S>:stats|<R>:restart|<C-^>:exit",
                state.iterations, 1000.0 / (elapsed.as_secs_f64() + *MIN_DELAY.read().unwrap() as f64), canvas.alive.len());
                stats_string.truncate(TERM_SIZE.read().unwrap().0 as usize - 1);
                stats_string =  format!("{}{}", &stats_string, ELEMENT_VISUALS[&components::Element::Wall].to_string().repeat((TERM_SIZE.read().unwrap().0 - stats_string.len() as u16) as usize));
                
                queue!(stdout, MoveTo(0, TERM_SIZE.read().unwrap().1), PrintStyledContent(stats_string.with(Color::Rgb { r: 255, g: 60, b: 70 }))).unwrap();
                
                stdout.flush().unwrap();
                last_refresh = Instant::now();
            }
        }
    }   
}