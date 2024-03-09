

use crossterm::style::Color;


use crate::TERM_SIZE;

#[derive(PartialEq, Clone, Hash, Eq, Copy)]
pub enum Species {
    NormalSnake,
    DetachedSnake,
    Wesp
}

#[derive(PartialEq, Clone, Copy)]
pub enum DirectionX {
    Left,
    Right,
    None,
}
#[derive(PartialEq, Clone, Copy)]
pub enum DirectionY {
    Up,
    Down,
    None,
}

pub fn directions_to_check(current_dir: &(DirectionX, DirectionY), fov: isize) -> Vec<(DirectionX, DirectionY)> {
    let circle = vec![
        (DirectionX::Left, DirectionY::None),
        (DirectionX::Left, DirectionY::Up),
        (DirectionX::None, DirectionY::Up),
        (DirectionX::Right, DirectionY::Up),
        (DirectionX::Right, DirectionY::None),
        (DirectionX::Right, DirectionY::Down),
        (DirectionX::None, DirectionY::Down),
        (DirectionX::Left, DirectionY::Down),
    ];

    let current_index = circle.iter().position(|&x| x == *current_dir).unwrap() as isize;
    let circle_len = circle.len() as isize;

    let mut directions_to_check = Vec::new();
    for i in -fov..=fov {
        // Move to the next index
        //   index = (index +1) % array.len(); loop through the array avoiding out of bounds
        //% operator is the remainder operator, not the modulo operator
        let index_circle = (((current_index + i)%circle_len) + circle_len)%circle_len;
        if i == 0 {
            directions_to_check.insert(0, circle[index_circle as usize])
        } else {
        directions_to_check.push(circle[index_circle as usize]);
        }
    }
    directions_to_check
}

pub fn pos_alteration_by_direction(dir_x: Option<&DirectionX>, dir_y: Option<&DirectionY>, position: &(u16, u16)) -> (i32, i32) {
    let mut new_position = (position.0 as i32, position.1 as i32);
    if let Some(dir_x) = dir_x {
        match dir_x {
            DirectionX::Left => new_position.0 -= 1,
            DirectionX::Right => new_position.0 += 1,
            DirectionX::None => (),
        }
    }
    if let Some(dir_y) = dir_y {
        match dir_y {
            DirectionY::Up => new_position.1 -= 1,
            DirectionY::Down => new_position.1 += 1,
            DirectionY::None => (),
        }
    }
    new_position
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
    WespHead,
    WespBody,
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
    pub(crate) species: Species,
    pub(crate) curent_direction: (DirectionX, DirectionY),
    pub(crate) spawner_at: (u16, u16),
    pub(crate) killed: bool,
}

impl Creature {
    pub(crate) fn move_to(&mut self, new_position: (u16, u16), moved_in_direction: (DirectionX, DirectionY), colision: bool) {
        if new_position.0 > TERM_SIZE.0 || new_position.0 <= 0 ||  new_position.1 > TERM_SIZE.1 || new_position.1 <= 0 || colision {
            self.killed = true;
            return;
        }
        let old_position = self.parts[0].position; // 0 Allways the head
        self.parts[0].position = new_position;
        self.curent_direction = moved_in_direction;
        
        let part_to_append = if self.species == Species::Wesp {
            Element::WespBody
        } else {
            match moved_in_direction {
                (DirectionX::None, DirectionY::Up) =>  Element::BodyPartVert,
                (DirectionX::Right,DirectionY::Up) =>  Element::BodyPartLeftLean,
                (DirectionX::Left, DirectionY::Up) =>  Element::BodyPartRightLean,
                (DirectionX::None, DirectionY::Down) =>Element::BodyPartVert,
                (DirectionX::Right,DirectionY::Down) =>Element::BodyPartRightLean,
                (DirectionX::Left, DirectionY::Down) =>Element::BodyPartLeftLean,
                (DirectionX::Right,DirectionY::None) =>Element::BodyPartHori,
                (DirectionX::Left, DirectionY::None) =>Element::BodyPartHori,
                _ => Element::BodyPartHori,
            }
        };
        self.parts.append(&mut vec![Part { element: part_to_append,
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
        Element::BodyPartVert | Element::BodyPartHori | Element::BodyPartLeftLean | Element::BodyPartRightLean | Element::WespBody => {
            if let Some(index) = creature_index {
                let color = self.alive[index].color;
                self.alive[index].parts.push(Part { element: new_element, position, color: color });
            } else {
                panic!("No creature index provided");
            }
        }
        _ => panic!("Creature heads may not be added with this function. Use add_creature instead."),
    }
}
    pub(crate) fn add_creature(&mut self, position: (u16, u16), color: Color, curent_direction: (DirectionX, DirectionY), species: Species, spawner_at: (u16, u16)) {
        let head = match species {
            Species::Wesp => Element::WespHead,
            _ => Element::BodyPartHead,
        };
            
        self.alive.push(Creature {parts: vec![Part { element: head, position: position, color: color }],
                                 color: color, curent_direction: curent_direction, killed: false,
                                 species: species, spawner_at: spawner_at});
    }

    pub(crate) fn unify_elements(&self) -> Vec<&Part> {
        let unified_elements = Vec::from_iter(self.environment.iter()
                        .chain(self.interactable.iter())
                        .chain(self.alive.iter().flat_map(|creature| creature.parts.iter())));
        unified_elements
    }
}

pub struct CanvasState {
    pub(crate) iterations: u128,
    pub(crate) cleared_coords: Vec<(u16, u16)>,
    //pub(crate) food_rate: u8,
}
