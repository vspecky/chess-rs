use ggez::graphics::Image;
use std::fmt;

pub struct Piece {
    pub img: Image,
    label: char,
}

impl Piece {
    pub fn new(img: Image, label: char) -> Self {
        Self { img, label }
    }
}
