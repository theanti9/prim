use std::{collections::HashMap, hash::BuildHasherDefault, marker::PhantomData};

use bevy_ecs::{
    prelude::{Component, Entity},
    query::{Added, Changed, Or, With, Without},
    schedule::SystemSet,
    system::{Commands, Query, Res},
};
use glam::Vec2;
use hashers::fx_hash::FxHasher;

use crate::instance::Instance2D;

#[derive(Component)]
pub struct Collidable;

#[derive(Component)]
pub struct Collider<T>
where
    T: Send + Sync + 'static,
{
    phantom: PhantomData<T>,
}

impl<T> Collider<T>
where
    T: Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self {
            phantom: PhantomData::<T>,
        }
    }
}

#[derive(Component)]
pub struct CollidesWith<T>
where
    T: Send + Sync + 'static,
{
    phantom: PhantomData<T>,
}

impl<T> CollidesWith<T>
where
    T: Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self {
            phantom: PhantomData::<T>,
        }
    }
}

pub struct HashGrid {
    pub size: i32,
}

#[derive(Component)]
pub struct HashMarker((i32, i32));
impl HashMarker {
    pub fn get_with_neighbors(&self) -> Vec<(i32, i32)> {
        Vec::from([
            self.0,
            (self.0 .0, self.0 .1 + 1),
            (self.0 .0, self.0 .1 - 1),
            (self.0 .0 + 1, self.0 .0),
            (self.0 .0 - 1, self.0 .0),
            (self.0 .0 + 1, self.0 .0 - 1),
            (self.0 .0 + 1, self.0 .0 + 1),
            (self.0 .0 - 1, self.0 .0 - 1),
            (self.0 .0 - 1, self.0 .0 + 1),
        ])
    }
}

pub fn update_hash_marker(
    mut collider_query: Query<
        (&mut HashMarker, &Instance2D),
        (
            With<Collidable>,
            Or<(Changed<Instance2D>, Added<Instance2D>)>,
        ),
    >,
    hash_grid: Res<HashGrid>,
) {
    collider_query.par_for_each_mut(512, |(mut hash_marker, inst)| {
        hash_marker.0 = inst.position.current_hash_grid(hash_grid.size);
    });
}

pub fn insert_hash_marker(
    q: Query<(Entity, &Instance2D), (With<Collidable>, Without<HashMarker>)>,
    hash_grid: Res<HashGrid>,
    mut commands: Commands,
) {
    for (entity, inst) in &q {
        commands
            .entity(entity)
            .insert(HashMarker(inst.position.current_hash_grid(hash_grid.size)));
    }
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Colliding<T>(pub Vec<Entity>, PhantomData<T>);

pub fn collisions<T>(
    collider_query: Query<(Entity, &Instance2D, &HashMarker), With<Collider<T>>>,
    collide_with_query: Query<(Entity, &Instance2D, &HashMarker), With<CollidesWith<T>>>,
    mut commands: Commands,
) where
    T: Send + Sync + 'static,
{
    let mut m: HashMap<(i32, i32), Vec<(Entity, Instance2D)>, BuildHasherDefault<FxHasher>> =
        HashMap::with_capacity_and_hasher(1000, BuildHasherDefault::<FxHasher>::default());
    for (entity, inst, hash_marker) in &collide_with_query {
        m.entry(hash_marker.0)
            .and_modify(|v| v.push((entity, *inst)))
            .or_insert(Vec::from([(entity, *inst)]));
    }

    for (entity, inst, hash_marker) in &collider_query {
        let mut collisions = Vec::new();
        for marker in hash_marker.get_with_neighbors() {
            if let Some(possible_collisions) = m.get(&marker) {
                collisions.extend(
                    possible_collisions
                        .iter()
                        .filter(|(_entity, inst_b)| overlapping(inst, inst_b))
                        .map(|(entity, _)| *entity),
                );
            }
        }
        if collisions.is_empty() {
            commands.entity(entity).remove::<Colliding<T>>();
        } else {
            commands
                .entity(entity)
                .insert(Colliding(collisions, PhantomData::<T>));
        }
    }
}

pub fn base_collision_detection() -> SystemSet {
    SystemSet::new()
        .with_system(update_hash_marker)
        .label("collision_update")
        .with_system(insert_hash_marker)
}

pub fn collision_system_set<T>() -> SystemSet
where
    T: Send + Sync + 'static,
{
    SystemSet::new()
        .with_system(collisions::<T>)
        .after("collision_update")
}

fn overlapping(a: &Instance2D, b: &Instance2D) -> bool {
    let a_x1 = a.position.x - a.scale.x / 2.0;
    let a_x2 = a.position.x + a.scale.x / 2.0;
    let b_x1 = b.position.x - b.scale.x / 2.0;
    let b_x2 = b.position.x + b.scale.x / 2.0;

    let a_y1 = a.position.y - a.scale.y / 2.0;
    let a_y2 = a.position.y + a.scale.y / 2.0;
    let b_y1 = b.position.y - b.scale.y / 2.0;
    let b_y2 = b.position.y + b.scale.y / 2.0;

    a_x1 < b_x2 && a_x2 > b_x1 && a_y2 > b_y1 && a_y1 < b_y2
}

trait HashGridVec {
    fn current_hash_grid(&self, grid_size: i32) -> (i32, i32);
}

impl HashGridVec for Vec2 {
    fn current_hash_grid(&self, grid_size: i32) -> (i32, i32) {
        (
            round_to_nearest(self.x, grid_size),
            round_to_nearest(self.y, grid_size),
        )
    }
}

fn round_to_nearest(i: f32, incr: i32) -> i32 {
    let mut res = (i as i32).abs() + incr / 2;
    res -= res % incr;
    if i < 0.0 {
        -res
    } else {
        res
    }
}
