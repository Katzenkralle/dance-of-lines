
use crossterm::style::Color;
use lazy_static::lazy_static;
use rand::Rng;
use std::collections::HashMap;

use crate::components::{self, Part};
const SIGHT_RADIUS: u16 = 3;

lazy_static! {
    static ref CHECK_DIRECTIONS: HashMap<&'static str, [&'static str; 2]> = HashMap::from([
        ("t _", ["t r", "t l"]),
        ("t r", ["t _", "_ r"]),
        ("t l", ["t _", "_ l"]),
        ("b _", ["b r", "b l"]),
        ("b r", ["b _", "_ r"]),
        ("b l", ["b _", "_ l"]),
        ("_ r", ["t r", "b r"]),
        ("_ l", ["t r", "t l"]),
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

pub fn spawner_handle(canvas: &mut Vec<components::Part>) {
    // Collect the indices of active spawns
    let active_spawn_indices: Vec<usize> = canvas
        .iter_mut()
        .enumerate()
        .filter(|(_, elem)| elem.color == Color::Rgb { r: 10, g: 255, b: 10 })
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

fn recursive_colision_check(parts_in_sight: &Vec<Part>, position: &(u16, u16), direction: &str, iterations_left: u8) -> u32{
    let x = direction.split_whitespace().take(2).collect::<Vec<&str>>();
    let dir_x:&str = x[0];
    let dir_y:&str = x[1];
    let mut dyn_pos_res: Vec<(i32, i32, u32)> = Vec::new();
    dyn_pos_res.push((i32::from(position.0) + DIRECTIONS[dir_x], i32::from(position.1) + DIRECTIONS[dir_y], 1));
    for dynamic_pos in CHECK_DIRECTIONS[direction].into_iter() {
        let x = dynamic_pos.split_whitespace().take(2).collect::<Vec<&str>>();
        dyn_pos_res.push((i32::from(position.0) + DIRECTIONS[x[0]], i32::from(position.1) + DIRECTIONS[x[1]], 1));
    }
    
    for (x, y, val) in &dyn_pos_res {
        for part in parts_in_sight {
           if part.position == (*x as u16, *y as u16) {
               //When element in sight, part is element in sight

           } 

        }   
    }
    dyn_pos_res.sort_by(|a, b| b.2.cmp(&a.2));
    dyn_pos_res[0].2
    /*In this example, the sort_by method is used to sort the vector data. 
    The closure provided to sort_by compares tuples (i32, i32, u32) based on the third element (u32).
    b.2.cmp(&a.2) compares the third element of b and a (in reverse order because we want the highest element first). */
}


pub fn head_handle(canvas: &mut Vec<components::Part>) {
    // Find all elements that head can see
    // Find all elements that head can see
    for head_index in 0..canvas.len() {
        if canvas[head_index].element.visual == '█' {
            let parts_in_sight: Vec<components::Part> = canvas.iter() 
                .enumerate()
                .filter(|(_, elem)| 
                    (elem.position.0 as i64 - canvas[head_index].position.0 as i64).pow(2) + 
                    (elem.position.1 as i64 - canvas[head_index].position.1 as i64).pow(2) <= SIGHT_RADIUS.pow(2) as i64
                )
                .map(|(_, elem)| elem.copy())
                .collect();

            let res = recursive_colision_check(&parts_in_sight, &canvas[head_index].position , &canvas[head_index].direction.to_string() , 3);
    }
}
}