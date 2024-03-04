
use crossterm::style::Color;
use lazy_static::lazy_static;
use rand::Rng;
use std::collections::HashMap;

use crate::components::{self, Part};
const SIGHT_RADIUS: u16 = 5;

lazy_static! {
    static ref CHECK_DIRECTIONS: HashMap<&'static str, ([&'static str; 3], &'static str ), > = HashMap::from([
        ("t _", (["t _", "t r", "t l"], "body_part_h")),
        ("t r", (["t r", "t _", "_ r"], "body_part_l")),
        ("t l", (["t l", "t _", "_ l"], "body_part_r")),
        ("b _", (["b _", "b r", "b l"], "body_part_h")),
        ("b r", (["b r", "b _", "_ r"], "body_part_r")),
        ("b l", (["b l", "b _", "_ l"], "body_part_l")),
        ("_ r", (["_ r", "t r", "b r"], "body_part_v")),
        ("_ l", (["_ l", "t l", "b l"], "body_part_v")),
    ]);
}

lazy_static!{
    static ref DIRECTIONS: HashMap<&'static str, i32> = HashMap::from([
        ("t", -1),
        ("b",  1),
        ("r",  1),
        ("l", -1),
        ("_",  0)
    ]);
}



fn check_collision(canvas: &Vec<components::Part>, position: (u16, u16)) -> bool {
    canvas.iter().filter(|elem| elem.position == position).count() > 0
}

fn get_unused_color(canvas: &Vec<components::Part>) -> Color {
    let mut color: Color = Color::Rgb { r: 0, g: 0, b: 0 };
    let mut color_used: bool = true;
    let mut rand_gen = rand::thread_rng();
    while color_used {
        color = Color::Rgb { r: rand_gen.gen_range(0..255), g: rand_gen.gen_range(0..255), b: rand_gen.gen_range(0..255) };
        color_used = canvas.iter().filter(|elem| elem.color == color).count() > 0;
    }
    color
}

pub fn spawner_handle(canvas: &mut Vec<components::Part>, heads: &i32) {
    let inactive_spawns: Vec<usize> = canvas.iter()
    .enumerate()
    .filter(|(_, elem)| elem.color == Color::Rgb { r: 10, g: 100, b: 10 }  && elem.element.e_type == "spawner")
    .map(|(index, _)| index)
    .collect();

    if *heads < inactive_spawns.len() as i32{
        canvas[inactive_spawns[0]].color = Color::Rgb { r: 10, g: 255, b: 10 };
    }

    
    // Collect the indices of active spawns
    let active_spawn_indices: Vec<usize> = canvas
        .iter_mut()
        .enumerate()
        .filter(|(_, elem)| elem.color == Color::Rgb { r: 10, g: 255, b: 10 } && elem.element.e_type == "spawner")
        .map(|(index, _)| index)
        .collect();

    let mut rand_gen = rand::thread_rng();

    for &index in &active_spawn_indices {
        let x: u8 = rand_gen.gen_range(0..100);
        if rand_gen.gen_range(0..100) < 10 {
            let pos: (u16, u16, &str) = match x % 4 {
                // 0: up 1: right 2: down 3: left
                0 => (canvas[index].position.0, canvas[index].position.1 - 1, "t _"),
                1 => (canvas[index].position.0 + 1, canvas[index].position.1, "_ r"),
                2 => (canvas[index].position.0, canvas[index].position.1 + 1, "b _"),
                3 => (canvas[index].position.0 - 1, canvas[index].position.1, "_ l"),
                _ => (1, 1, "b _"),
            };
            if !check_collision(canvas, (pos.0, pos.1)) {
                canvas.push(components::Part::new("body_part_head", (pos.0, pos.1), get_unused_color(canvas)).unwrap());
                canvas.last_mut().unwrap().direction = pos.2.to_string();
                canvas[index].color = Color::Rgb { r: 10, g: 100, b: 10 };
            }
        }
    }

}

fn recursive_colision_check(color: &Color, parts_in_sight: &Vec<Part>, position: &(u16, u16), direction: &str, iterations_left: u8) -> (i32, i32, &'static str , i64){
    let mut dyn_pos_res: Vec<(i32, i32, &str , i64)> = Vec::new();

    // Create a vector of tuples with the possible positions and their values(liklihood of beeing chosen)
    for direction_to_check in CHECK_DIRECTIONS[direction].0.into_iter() {
        let x = direction_to_check.split_whitespace().take(2).collect::<Vec<&str>>();
        dyn_pos_res.push((i32::from(position.0) + DIRECTIONS[x[0]], i32::from(position.1) + DIRECTIONS[x[1]], direction_to_check , 0));
    }
    
    // Iterate over the vector and check if the position is in sight, what value the sight has
    for index in 0..dyn_pos_res.len() {
        let (x, y, v_direction, val): &mut (i32, i32, &str, i64) = &mut dyn_pos_res[index];
        for part in parts_in_sight {
            if part.position == (*x as u16, *y as u16) {
                //When element in sight, part is element in sight
                if part.element.e_type == "alive"  {//&& part.color != *color
                    *val -= 100;
                }
                else if part.element.e_type == "food" {
                    *val += 100;
                }
                else if part.element.e_type == "wall" {
                    *val -= 10000;
                }
            } 
        }
        if iterations_left > 0 {
            *val += recursive_colision_check(color, &parts_in_sight, &(*x as u16, *y as u16), &v_direction, iterations_left - 1).3;
        }
        //dyn_pos_res[index].3 = *val;
    }
    
    dyn_pos_res.sort_by(|a, b| b.3.cmp(&a.3));
    dyn_pos_res[0]
    /*In this example, the sort_by method is used to sort the vector data. 
    The closure provided to sort_by compares tuples (i32, i32, u32) based on the third element (u32).
    b.2.cmp(&a.2) compares the third element of b and a (in reverse order because we want the highest element first). */
}

pub fn head_handle(canvas: &mut Vec<components::Part>, head_count: &mut i32) {
    // Find all elements that head can see
    let mut heads = 0;
    for head_index in 0..canvas.len() {
        if canvas[head_index].element.visual == '█' {
            // Keep track on how many heads are there
            heads += 1;

            let parts_in_sight: Vec<components::Part> = canvas.iter() 
                .enumerate()
                .filter(|(_, elem)| 
                    (elem.position.0 as i64 - canvas[head_index].position.0 as i64).pow(2) + 
                    (elem.position.1 as i64 - canvas[head_index].position.1 as i64).pow(2) <= SIGHT_RADIUS.pow(2) as i64
                )
                .map(|(_, elem)| elem.copy())
                .collect();

            let res = recursive_colision_check(&canvas[head_index].color, &parts_in_sight, &canvas[head_index].position , &canvas[head_index].direction.to_string() , 3);
            let old_pos: (u16, u16) = canvas[head_index].position;           
            
            // Check colisions of new position
            let colision = canvas.iter().filter(|elem| elem.position == (res.0 as u16, res.1 as u16))
            .filter(|elem| elem.element.e_type != "food").count() > 0;
            
            // Move the head to the new position, spawn a new body part and update the direction
            canvas[head_index].direction = res.2.to_string();
            canvas[head_index].move_to((res.0 as u16, res.1 as u16), colision);
            canvas.push(components::Part::new(CHECK_DIRECTIONS[res.2].1, old_pos, canvas[head_index].color).unwrap());

            // Clean up food
            let mut to_remove: Vec<usize> = Vec::new();
            // Iterate over a mutable reference to the canvas vector
            for (index, element) in canvas.iter_mut().enumerate() {
                if element.position == (res.0 as u16, res.1 as u16) && element.element.e_type == "food" {
                    to_remove.push(index);
                } 
            }
            
            // Remove elements from the canvas vector based on the condition
            to_remove.iter().for_each(|x| {canvas.remove(*x);});
        }
    }
    *head_count = heads;
}

pub fn handle_killed(canvas: &mut Vec<components::Part>) {
    let to_remove: Vec<usize> = canvas.iter().enumerate().filter(|(_, elem)| elem.killed).map(|(index, _)| index).collect();
    for index in &to_remove {
        for mod_y in -1..=1i32 {
            for mod_x in -1..=1i32 {
                canvas.iter()
                .enumerate()
                .filter(|(_, elem)| elem.color == canvas[*index].color)
                .filter(|(_, elem)| elem.position == ((canvas[*index].position.0.try_into().unwrap_or_else(|_| 2) + mod_x).try_into().unwrap_or_else(|_| 0), 
                (canvas[*index].position.1.try_into().unwrap_or_else(|_| 2) + mod_y).try_into().unwrap_or_else(|_| 0)))
                .map(|(index, _)| index)
                .collect::<Vec<usize>>().iter().for_each(|x| canvas[*x].killed = true);
            }
        }
    }
    // Must be hear else the indices will be wrong
    for index in to_remove.iter().rev() {
        canvas.remove(*index);
    }
}

pub fn spawn_food(canvas: &mut Vec<components::Part>) {
    let mut rng = rand::thread_rng();

    if rng.gen_bool(0.2){
        let mut pos: (u16, u16) = (rng.gen_range(1..crate::TERM_SIZE.0 - 1), rng.gen_range(1..crate::TERM_SIZE.1 - 1));
        while check_collision(canvas, pos) {
            pos = (rng.gen_range(1..crate::TERM_SIZE.0 - 1), rng.gen_range(1..crate::TERM_SIZE.1 - 1));
        }
        canvas.push(components::Part::new("food", pos, get_unused_color(canvas)).unwrap());
    }
}