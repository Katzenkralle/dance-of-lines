use std::thread;
use std::sync::mpsc;
use crossterm::style::Color;
use lazy_static::lazy_static;
use rand::seq::index;
use rand::Rng;
use crate::HashMap;

use crate::components::{self, CanvasParts, Creature, Direction, Element, Part, Species};
const SIGHT_RADIUS: u16 = 4;

lazy_static! {
    pub static ref CHECK_DIRECTIONS: HashMap<(Direction, Direction), (Vec<Vec<Direction>>, Element)> = HashMap::from([
        ((Direction::Up, Direction::None), (vec![vec![Direction::Up, Direction::None], vec![Direction::Up, Direction::Right], vec![Direction::Up, Direction::Left]], Element::BodyPartHori)),
        ((Direction::Up, Direction::Right), (vec![vec![Direction::Up, Direction::Right], vec![Direction::Up, Direction::None], vec![Direction::None, Direction::Right]], Element::BodyPartLeftLean)),
        ((Direction::Up, Direction::Left), (vec![vec![Direction::Up, Direction::Left], vec![Direction::Up, Direction::None], vec![Direction::None, Direction::Left]], Element::BodyPartRightLean)),
        ((Direction::Down, Direction::None), (vec![vec![Direction::Down, Direction::None], vec![Direction::Down, Direction::Right], vec![Direction::Down, Direction::Left]], Element::BodyPartHori)),
        ((Direction::Down, Direction::Right), (vec![vec![Direction::Down, Direction::Right], vec![Direction::Down, Direction::None], vec![Direction::None, Direction::Right]], Element::BodyPartRightLean)),
        ((Direction::Down, Direction::Left), (vec![vec![Direction::Down, Direction::Left], vec![Direction::Down, Direction::None], vec![Direction::None, Direction::Left]], Element::BodyPartLeftLean)),
        ((Direction::None, Direction::Right), (vec![vec![Direction::None, Direction::Right], vec![Direction::Up, Direction::Right], vec![Direction::Down, Direction::Right]], Element::BodyPartVert)),
        ((Direction::None, Direction::Left), (vec![vec![Direction::None, Direction::Left], vec![Direction::Up, Direction::Left], vec![Direction::Down, Direction::Left]], Element::BodyPartVert)),
    ]);
}

lazy_static!{
    static ref DIRECTIONS: HashMap<components::Direction, i32> = HashMap::from([
        (Direction::Up, -1),
        (Direction::Down,  1),
        (Direction::Right,  1),
        (Direction::Left, -1),
        (Direction::None,  0)
    ]);
}



fn check_collision(canvas: &Vec<Part>, position: (u16, u16)) -> bool {
    canvas.iter().filter(|elem| elem.position == position).count() > 0
}

fn get_unused_color(creatures: &Vec<Creature>) -> Color {
    let mut color: Color = Color::Rgb { r: 0, g: 0, b: 0 };
    let mut color_used: bool = true;
    let mut rand_gen = rand::thread_rng();
    while color_used {
        color = Color::Rgb { r: rand_gen.gen_range(0..255), g: rand_gen.gen_range(0..255), b: rand_gen.gen_range(0..255) };
        color_used = creatures.iter().filter(|cret| cret.color == color).count() > 0;
    }
    color
}

pub fn spawner_handle(canvas: &mut CanvasParts) {
    let inactive_spawns: Vec<usize> = canvas.interactable.iter()
    .enumerate()
    .filter(|(_, elem)| elem.element == Element::Spawn && canvas.alive.iter().filter(|creature| creature.spawner_at == elem.position).count() == 0)
    .map(|(index, _)| index)
    .collect();

    if canvas.alive.len()< inactive_spawns.len() {
        canvas.interactable[inactive_spawns[0]].color = Color::Rgb { r: 10, g: 255, b: 10 };
    }

    
    // Collect the indices of active spawns
    let active_spawn_indices: Vec<usize> = canvas.interactable.iter_mut()
        .enumerate()
        .filter(|(_, elem)| elem.element == Element::Spawn && canvas.alive.iter().filter(|creature| creature.spawner_at == elem.position).count() == 0)
        .map(|(index, _)| index)
        .collect();

    let mut rand_gen = rand::thread_rng();
    //let enviorment = canvas.environment.clone();
    for &index in &active_spawn_indices {
        let x: u8 = rand_gen.gen_range(0..100);
        if rand_gen.gen_range(0..100) < 10 {
            let pos: (u16, u16, (Direction, Direction)) = match x % 4 {
                // 0: up 1: right 2: down 3: left
                0 => (canvas.interactable[index].position.0, canvas.interactable[index].position.1 - 1, (Direction::Up, Direction::None)),
                1 => (canvas.interactable[index].position.0 + 1, canvas.interactable[index].position.1, (Direction::None, Direction::Right)),
                2 => (canvas.interactable[index].position.0, canvas.interactable[index].position.1 + 1, (Direction::Down, Direction::None)),
                3 => (canvas.interactable[index].position.0 - 1, canvas.interactable[index].position.1, (Direction::None, Direction::Left)),
                _ => (1, 1, (Direction::Down, Direction::None)),
            };
            
            if !check_collision(&canvas.environment, (pos.0, pos.1)) {
                canvas.add_creature((pos.0, pos.1), get_unused_color(&canvas.alive), pos.2, Species::NormalSnake, canvas.interactable[index].position);
                canvas.interactable[index].color = Color::Rgb { r: 10, g: 100, b: 10 };
            }
        }
    }

}

fn recursive_colision_check(color: &Color, parts_in_sight: &Vec<Part>, position: &(u16, u16),
                             direction: &(Direction, Direction), iterations_left: u8) -> (i32, i32, (Direction, Direction), i64){
    // Position to check x, Position to check y, Direction to walk, Vale of Direction
    let mut dyn_pos_res: Vec<(i32, i32, (Direction, Direction), i64)> = Vec::new();


    // Create a vector of tuples with the possible positions and their values(liklihood of beeing chosen)
    for direction_to_check in CHECK_DIRECTIONS[&direction].0.iter() {
        let (dir_x, dir_y): (Direction, Direction) = (direction_to_check[0], direction_to_check[1]);
        dyn_pos_res.push((i32::from(position.0) + DIRECTIONS[&dir_x], i32::from(position.1) + DIRECTIONS[&dir_y], (dir_x, dir_y), 0));
    }

    let (sender, receiver) = mpsc::channel();
    let mut threads: Vec<thread::JoinHandle<()>> = Vec::new();
    // Iterate over the vector and check if the position is in sight, what value the sight has
    for index in 0..dyn_pos_res.len() {
        let (x, y, v_direction, mut val): (i32, i32, (Direction, Direction), i64) = dyn_pos_res[index].clone();
        for part in parts_in_sight {
            if part.position == (x as u16, y as u16) {
                //When element in sight, part is element in sight
                val = match part.element {
                        Element::Wall | Element::Spawn => -100,
                        Element::BodyPartHori | Element::BodyPartVert |
                        Element::BodyPartLeftLean | Element::BodyPartRightLean | Element::BodyPartHead => val -100,
                        Element::Food => 10,
                        _ => 0,
                    }
            } 
        }
        dyn_pos_res[index].3 = val;

        if iterations_left > 0 {
            let sender = sender.clone();
            let color = color.clone();
            let parts_in_sight = parts_in_sight.clone();
            threads.push(thread::spawn(move || {
                let t_val = recursive_colision_check(&color, &parts_in_sight, &(x as u16, y as u16), &v_direction, iterations_left - 1).3;
                sender.send((t_val, index)).unwrap();
            }));
        }
    }

   
    for _ in 0..(threads.len()) {
        let thread_res = receiver.recv().unwrap();
        dyn_pos_res[thread_res.1].3 += thread_res.0;
    }

    
    dyn_pos_res.sort_by(|a, b| b.3.cmp(&a.3));
    dyn_pos_res[0]
    /*In this example, the sort_by method is used to sort the vector data. 
    The closure provided to sort_by compares tuples (i32, i32, u32) based on the third element (u32).
    b.2.cmp(&a.2) compares the third element of b and a (in reverse order because we want the highest element first). */
}

pub fn head_handle(canvas: &mut CanvasParts) {
    //Todo: Reomve or improve threading in this function!!
    // Find all elements that head can see
    let (sender, receiver) = mpsc::channel();

    let unified_canvas = canvas.clone();
    for creature in canvas.alive.iter_mut() {
        let head = creature.parts.iter().filter(|elem| elem.element == Element::BodyPartHead).next();
        if head.is_none() {
            continue;
        }
        let head = head.unwrap().clone(); // Clone the head to avoid borrowing issues
            

        // Concartination of all parts in sight
        let parts_in_sight: Vec<Part> = unified_canvas.unify_elements().iter()
                 .filter(|elem| 
                (elem.position.0 as i64 - head.position.0 as i64).pow(2) + 
                (elem.position.1 as i64 - head.position.1 as i64).pow(2) <= SIGHT_RADIUS.pow(2) as i64)
                .map(|elem| **elem)
                .collect::<Vec<_>>();    

        let parts_in_sight_thread = parts_in_sight.clone();
        let dir_thread = creature.curent_direction.clone();
        let sender = sender.clone();

        let thread = thread::spawn(move || {
                let result: (i32, i32, (Direction, Direction), i64) = recursive_colision_check(&head.color, &parts_in_sight_thread,
                                   &head.position, &dir_thread , SIGHT_RADIUS as u8);   
                sender.send(result).unwrap();
        });

        let res: (i32, i32, (Direction, Direction), i64) = receiver.recv().unwrap();
        thread.join().unwrap();
        // Check colisions of new position
        let colision = parts_in_sight.iter().filter(|elem| elem.position == (res.0 as u16, res.1 as u16))
                        .filter(|elem| elem.element != Element::Food).count() > 0;
        
        // Move the head to the new position, spawn a new body part and update the direction
        creature.move_to((res.0 as u16, res.1 as u16), res.2, colision);

        // Clean up food
        let mut to_remove: Vec<usize> = Vec::new();
        // Iterate over a mutable reference to the canvas vector
        for (index, element) in canvas.interactable.iter_mut().enumerate() {
            if element.position == (res.0 as u16, res.1 as u16) && element.element == Element::Food {
                to_remove.push(index);
            } 
        }
        
        // Remove elements from the canvas vector based on the condition
        to_remove.iter().for_each(|x| {canvas.interactable.remove(*x);});
    }
}


pub fn handle_killed(creatures: &mut Vec<Creature>, cleared_coords: &mut Vec<(u16, u16)>) {
    //let killed_creatures: Vec<usize> = creatures.iter().enumerate().filter(|(_, elem)| elem.killed).map(|(index, _)| index).collect::<Vec<_>>();
    let mut to_remove: Vec<usize> = Vec::new();
    for (index, creature) in creatures.iter_mut().enumerate().rev() {
        //if creature.species == Species::DetachedSnake && creature.parts.len() > 4{
         //   creatures[index].parts.remove(1);
        //}
        if creature.killed {
            for _ in 0..=3 {
                if creature.parts.len() == 0 {
                    to_remove.push(index);
                    break;
                }
                cleared_coords.push(creature.parts[0].position);
                creature.parts.remove(0);
            }
        }
    }
    // It is bad to remove elements from a vector while iterating over it
    to_remove.iter().for_each(|x| {creatures.remove(*x);});
}



pub fn spawn_food(canvas: &mut CanvasParts) {
    let mut rng = rand::thread_rng();

    if rng.gen_bool(0.4){
        let mut pos: (u16, u16) = (rng.gen_range(1..crate::TERM_SIZE.0 - 1), rng.gen_range(1..crate::TERM_SIZE.1 - 1));
        while check_collision(&canvas.unify_elements().iter().map(|part| **part).collect(), pos) {
            pos = (rng.gen_range(1..crate::TERM_SIZE.0 - 1), rng.gen_range(1..crate::TERM_SIZE.1 - 1));
        }
        canvas.add_element(Element::Food, pos, Some(get_unused_color(&canvas.alive)), None);
    }
}