
use crate::components::{DirectionX, DirectionY, Element, Part, Species, directions_to_check, pos_alteration_by_direction};
use crate::CanvasParts;
use std::thread;
use std::sync::mpsc;

fn snake_path_match(elem:Element) -> i64 {
    match elem {
        Element::Wall | Element::Spawn => -100,
        Element::BodyPartHori | Element::BodyPartVert | Element::WespBody | Element::WespHead |
        Element::BodyPartLeftLean | Element::BodyPartRightLean | Element::BodyPartHead => -100,
        Element::Food => 10,
    }
}

fn wesp_path_match(elem:Element) -> i64 {
    match elem {
        Element::Wall | Element::Spawn | Element::Food => -100,
        Element::WespBody => -20,
        Element::BodyPartHead => 30,
        _ => 1,
    }
}

fn recursive_colision_check(path_to_match: fn(Element) -> i64, parts_in_sight: &Vec<Part>, fov: isize, position: &(u16, u16),
                             direction: &(DirectionX, DirectionY), iterations_left: u8) -> (i32, i32, (DirectionX, DirectionY), i64){
    // Position to check x, Position to check y, Direction to walk, Vale of Direction
    let mut dyn_pos_res: Vec<(i32, i32, (DirectionX, DirectionY), i64)> = Vec::new();


    // Create a vector of tuples with the possible positions and their values(liklihood of beeing chosen)
    for direction_to_check in directions_to_check(&direction, fov).iter() {
        let (dir_x, dir_y): (DirectionX, DirectionY) = ((*direction_to_check).0, (*direction_to_check).1);
        let (x, y): (i32, i32) = pos_alteration_by_direction(Some(&dir_x), Some(&dir_y), position);
        dyn_pos_res.push((x, y, (dir_x, dir_y), 0));
    }

    let (sender, receiver) = mpsc::channel();
    let mut threads: Vec<thread::JoinHandle<()>> = Vec::new();
    // Iterate over the vector and check if the position is in sight, what value the sight has
    for index in 0..dyn_pos_res.len() {
        let (x, y, v_direction, mut val): (i32, i32, (DirectionX, DirectionY), i64) = dyn_pos_res[index].clone();
        for part in parts_in_sight {
            if part.position == (x as u16, y as u16) {
                //When element in sight, part is element in sight
                val = path_to_match(part.element)
            } 
        }
        dyn_pos_res[index].3 = val;

        if iterations_left > 0 {
            let sender = sender.clone();
            let parts_in_sight = parts_in_sight.clone();
            threads.push(thread::spawn(move || {
                let t_val = recursive_colision_check(path_to_match, &parts_in_sight, fov, &(x as u16, y as u16), &v_direction, iterations_left - 1).3;
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
    let mut wesp_kills: Vec<usize> = Vec::new();

    let cloned_canvas = canvas.clone();
    for creature in canvas.alive.iter_mut() {
        
        let (fov, speed, sight_radius): (isize, u8, i32) = match creature.species {
            Species::DetachedSnake => (2, 1, 4),
            Species::Wesp => (1, 2, 4),
            _ => (1, 1, 4),
        };

        for _ in 0..speed {
            let head = creature.parts.iter().filter(|elem| elem.element == Element::BodyPartHead || elem.element == Element::WespHead).next();
            if head.is_none() {
                continue;
            }
            let head = head.unwrap().clone(); // Clone the head to avoid borrowing issues
                

            // Concartination of all parts in sight
            let parts_in_sight: Vec<Part> = cloned_canvas.unify_elements().iter()
                    .filter(|elem| 
                    (elem.position.0 as i64 - head.position.0 as i64).pow(2) + 
                    (elem.position.1 as i64 - head.position.1 as i64).pow(2) <= sight_radius.pow(2) as i64)
                    .map(|elem| **elem)
                    .collect::<Vec<_>>();   

            let matcher: fn(Element) -> i64 = match creature.species {
                Species::DetachedSnake | Species::NormalSnake => snake_path_match,
                Species::Wesp => wesp_path_match,
            };
           
            let res = recursive_colision_check(matcher, &parts_in_sight, fov,
                            &head.position, &creature.curent_direction , sight_radius as u8);   

            // Check colisions of new position
            let mut colision = false;
            if creature.species == Species::Wesp {
                for part in parts_in_sight.iter() {
                    if part.position == (res.0 as u16, res.1 as u16) && part.element == Element::BodyPartHead{
                        wesp_kills.push(cloned_canvas.alive.iter().position(|x| x.parts[0].position == (res.0 as u16, res.1 as u16)).unwrap());
                    } else if part.position == (res.0 as u16, res.1 as u16) && part.element == Element::Food {
                        colision = true;
                    }
                }
            } else {
                colision = parts_in_sight.iter().any(|elem| elem.position == (res.0 as u16, res.1 as u16) && elem.element != Element::Food);
            }
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
    wesp_kills.iter().for_each(|x| {canvas.alive[*x].killed = true;});
}
