use crate::{instance::Instance2D, state::State, time::Time};

pub trait Updateable {
    fn update(&mut self, time: &Time, state: &State);
}

pub trait Renderable {
    fn get_instance(&self) -> Instance2D;
}
