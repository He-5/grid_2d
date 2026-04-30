use std::marker::PhantomData;
use crate::Position;

mod layout;

pub struct Grid<T> {
    // temp marker for inner layout
    layout: PhantomData<T>
}

impl <T> Grid<T> {
    // CURD
    pub fn get(&self, pos: &Position) -> Result<&T, ()> {
        todo!("get from layout")
    }

    pub fn get_mut(&mut self, pos: &Position) -> Result<&mut T, ()> {
        todo!("get mut ref from layout")
    }

    pub fn set(&mut self, pos: &Position, item: T) -> Result<Option<T>, ()> {
        todo!("set item to pos in layout")
    }

    pub fn rmv(&mut self, pos: &Position) -> Result<Option<T>, ()> {
        todo!("rmv item from pos in layout")
    }
}