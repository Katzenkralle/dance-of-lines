use crossterm::style::Color;
use rand::Rng;

use crate::components::{CanvasParts, Creature, DirectionX, DirectionY, Element, Part, Species};


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
    // Collect the indices of active spawns
    let unused_spawns: Vec<usize> = canvas.interactable.iter_mut()
        .enumerate()
        .filter(|(_, elem)| elem.element == Element::Spawn && 
                canvas.alive.iter()
                .filter(|creature| creature.spawner_at == elem.position).count() == 0)
        .map(|(index, elem)|  {elem.color = Color::Rgb { r: 10, g: 255, b: 10 }; index})
        .collect();

    let mut rand_gen = rand::thread_rng();
    let active_spawn_count = unused_spawns.len();

    let x: u16 = rand_gen.gen_range(0..=1000);
    if x < 10*active_spawn_count as u16 {
        let index = unused_spawns[rand_gen.gen_range(0..active_spawn_count)];
        let pos: (u16, u16, (DirectionX, DirectionY)) = match x % 4 {
            // 0: up 1: right 2: down 3: left
            0 => (canvas.interactable[index].position.0, canvas.interactable[index].position.1 - 1, (DirectionX::None, DirectionY::Up)),
            1 => (canvas.interactable[index].position.0 + 1, canvas.interactable[index].position.1, (DirectionX::Right,DirectionY::None)),
            2 => (canvas.interactable[index].position.0, canvas.interactable[index].position.1 + 1, (DirectionX::None, DirectionY::Down)),
            3 => (canvas.interactable[index].position.0 - 1, canvas.interactable[index].position.1, (DirectionX::Left, DirectionY::None)),
            _ => (1, 1, (DirectionX::None, DirectionY::Down)),
        };
        
        if !check_collision(&canvas.environment, (pos.0, pos.1)) {
            let color = get_unused_color(&canvas.alive);
            match x % 10 {
                0 | 1 => canvas.add_creature((pos.0, pos.1), color, pos.2, Species::DetachedSnake, canvas.interactable[index].position),
                2 => canvas.add_creature((pos.0, pos.1), color, pos.2, Species::Wesp, canvas.interactable[index].position),
                _ => canvas.add_creature((pos.0, pos.1), color, pos.2, Species::NormalSnake, canvas.interactable[index].position),
            }
            canvas.interactable[index].color = Color::Rgb { r: 10, g: 100, b: 10 };
            canvas.alive.sort_by(|a, b| {
                match (a.species, b.species) {
                    (_, Species::Wesp) => std::cmp::Ordering::Less,
                    (Species::Wesp, _) => std::cmp::Ordering::Greater,
                    _ => std::cmp::Ordering::Equal,
                }
            });   
        }
    }
    

}



pub fn handle_killed(creatures: &mut Vec<Creature>, cleared_coords: &mut Vec<(u16, u16)>) {
    //let killed_creatures: Vec<usize> = creatures.iter().enumerate().filter(|(_, elem)| elem.killed).map(|(index, _)| index).collect::<Vec<_>>();
    let mut to_remove: Vec<usize> = Vec::new();
    for (index, creature) in creatures.iter_mut().enumerate().rev() {
        
        if creature.species == Species::DetachedSnake && creature.parts.len() > 20 {
            cleared_coords.push(creature.parts[1].position); //0 Is head, 1 is oldes part
            creature.parts.remove(1);
        } else if creature.species == Species::Wesp && creature.parts.len() > 8 {
            cleared_coords.push(creature.parts[1].position); //
            cleared_coords.push(creature.parts[2].position);
            creature.parts.remove(1); //0 Is head, 1 is oldes part, tow parts are removed
            creature.parts.remove(1);
        }

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

    if rng.gen_bool(0.30){
        let mut pos: (u16, u16) = (rng.gen_range(1..crate::TERM_SIZE.0 - 1), rng.gen_range(1..crate::TERM_SIZE.1 - 1));
        while check_collision(&canvas.unify_elements().iter().map(|part| **part).collect(), pos) {
            pos = (rng.gen_range(1..crate::TERM_SIZE.0 - 1), rng.gen_range(1..crate::TERM_SIZE.1 - 1));
        }
        canvas.add_element(Element::Food, pos, Some(get_unused_color(&canvas.alive)), None);
    }
}