

use crossterm::style::Color;


use crate::TERM_SIZE;
use crate::part_handler;

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    None,
}

#[derive(PartialEq, Eq, Hash, Copy, Clone)]

pub enum Element {
    Wall,
    Spawn,
    BodyPartVert,
    BodyPartHori,
    BodyPartLeftLean,
    BodyPartRightLean,
    Food,
    BodyPartHead,
}
#[derive(Copy, Clone)]

pub struct Part {
    pub(crate) element: Element,
    pub(crate) position: (u16, u16),
    pub(crate) color: Color,
}


#[derive(Clone)]

pub(crate) struct Creature {
    pub(crate) parts: Vec<Part>,
    pub(crate) color: Color,
    pub(crate) curent_direction: (Direction, Direction),
    pub(crate) spawner_at: (u16, u16),
    pub(crate) killed: bool,
}

impl Creature {
    pub(crate) fn move_to(&mut self, new_position: (u16, u16), moved_in_direction: (Direction, Direction), colision: bool) {
        let old_position = self.parts[0].position; // 0 Allways the head
        self.parts[0].position = new_position;
        self.curent_direction = moved_in_direction;
        if new_position.0 > TERM_SIZE.0 || new_position.1 > TERM_SIZE.1 || colision {
            self.killed = true;
        }
        self.parts.append(&mut vec![Part { element: part_handler::CHECK_DIRECTIONS[&self.curent_direction].1,
                                             position: old_position, color: self.color }]);
    
}
}
#[derive(Clone)]
pub(crate) struct CanvasParts {
    pub(crate) alive: Vec<Creature>,
    pub(crate) environment: Vec<Part>,
    pub(crate) interactable: Vec<Part>,   
}

impl CanvasParts{
    pub(crate) fn add_element(&mut self, new_element: Element, position: (u16, u16), color: Option<Color>,
    creature_index: Option<usize>){
    if position.0 > TERM_SIZE.0 || position.1 > TERM_SIZE.1 {
        return;
    }
    
    match new_element {
        Element::Wall => self.environment.push(Part { element: new_element, position, color: color.unwrap() }),
        Element::Spawn | Element::Food => self.interactable.push(Part { element: new_element, position, color: color.unwrap() }),
        Element::BodyPartHead | Element::BodyPartVert | Element::BodyPartHori | Element::BodyPartLeftLean | Element::BodyPartRightLean => {
            if let Some(index) = creature_index {
                let color = self.alive[index].color;
                self.alive[index].parts.push(Part { element: new_element, position, color: color });
            } else {
                panic!("No creature index provided");
            }
        }
    }
}
    pub(crate) fn add_creature(&mut self, position: (u16, u16), color: Color, curent_direction: (Direction, Direction), spawner_at: (u16, u16)) {
        self.alive.push(Creature {parts: vec![Part { element: Element::BodyPartHead, position: position, color: color }],
                                 color: color, curent_direction: curent_direction, killed: false, spawner_at: spawner_at});
    }

    pub(crate) fn unify_elements(&self) -> Vec<&Part> {
        let unified_elements = Vec::from_iter(self.environment.iter()
                        .chain(self.interactable.iter())
                        .chain(self.alive.iter().flat_map(|creature| creature.parts.iter())));
        unified_elements
    }
}

pub struct CanvasState {
    pub(crate) colors: i32,
    pub(crate) iterations: u128,
    //pub(crate) food_rate: u8,
}
