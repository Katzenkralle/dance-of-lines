

use crossterm::style::Color;
use crate::TERM_SIZE;
use rand::Error;

pub(crate) struct Element {
    pub(crate) visual: char,
    pub(crate) e_type: &'static str,
}

impl Element {
    pub(crate) fn new(name: &str) -> Result<Element, Error> {
        match name {
            "wall" => Ok(Element { visual: '∩', e_type: "blob" }),
            "spawn" => Ok(Element { visual: '@', e_type: "changing" }),
            "body_part_v" => Ok(Element { visual: '|', e_type: "alive" }),
            "body_part_h" => Ok(Element { visual: '-', e_type: "alive" }),
            "body_part_l" => Ok(Element { visual: '/', e_type: "alive" }),
            "body_part_r" => Ok(Element { visual: '\\', e_type: "alive" }),
            "body_part_m+" => Ok(Element { visual: '+', e_type: "alive" }),
            "body_part_mx" => Ok(Element { visual: 'X', e_type: "alive" }),
            "food" => Ok(Element { visual: '#', e_type: "changing" }),
            "body_part_head" => Ok(Element { visual: '█', e_type: "alive" }),
            _ => Err(Error::new("Invalid element")),
        }
    }
    pub(crate) fn copy(&self) -> Element {
        Element {
            visual: self.visual,
            e_type: self.e_type,
        }
    }
}

pub struct Part {
    pub(crate) element: Element,
    pub(crate) position: (u16, u16),
    pub(crate) color:  Color,
    pub(crate) killed: bool,
    pub(crate) direction: std::string::String, //"t/b/_ l/r/_"
}

impl Part {
    pub(crate) fn new(element: &str, position: (u16, u16), color: Color) -> Result<Part,Error> {
        if  position.0 > TERM_SIZE.0 || position.1 > TERM_SIZE.1 {
            // < 0 not needet because u16 is always positive
            return Err(Error::new("Invalid position"));
        }
        Ok(Part {
            element: Element::new(element)?,
            position,
            color: color,
            direction: String::from("_ _"),
            killed: false,
        })
    }
    pub(crate) fn copy(&self) -> Part {
        Part {
            element: self.element.copy(),
            position: self.position,
            color: self.color,
            killed: self.killed,
            direction: self.direction.clone(),
        }
    }
}

pub(crate) struct CanvasState {
    pub(crate) canvas: Vec<Part>,
    pub(crate) prev_canvas: Vec<Part>,
    pub(crate) iterations: u128,
    pub(crate) food_rate: u8,
}

impl CanvasState {
    pub(crate) fn new(canvas: Vec<Part>) -> CanvasState {
        CanvasState {
            canvas: canvas,
            prev_canvas: Vec::new(),
            iterations: 0,
            food_rate: 10,
        }
    }
    pub(crate) fn update(self, new_canvas: Vec<Part>) -> CanvasState {
        CanvasState {
            canvas: new_canvas,
            prev_canvas: self.canvas,
            food_rate: self.food_rate,
            iterations: self.iterations + 1,
        }
    }
}

