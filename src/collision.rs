//! Defines a hash grid for sectioning collision detection to avoid comparing all collidable entities with all other collidable entities.
//!
//! Marker types are also used to define what entities can collide with each other. An entity with `Collider<T>` will be marked as having collided
//! with any entities that are overlapping which have a `CollidesWith<T>` for the same `T`. The entity with the `Collider<T>` will have a `Colliding<T>`
//! component added when it is overlapping with any of the `CollidesWith<T>` entities. This means that collisions are not bi-directional by default.
//!
//! Each collidable type needs to have separate systems set up using `collision_system_set<T>()`. These should be added to the `pre_update` stage,
//! to ensure movements from the last frame and their resulting collisions are present for all systems during the current frame.
use std::{collections::HashMap, hash::BuildHasherDefault, marker::PhantomData};

use bevy_ecs::{
    prelude::{Component, Entity},
    query::{Added, Changed, Or, With, Without},
    schedule::SystemSet,
    system::{Commands, Query, Res},
};
use glam::Vec2;
use hashers::fx_hash::FxHasher;

use crate::{instance::Instance2D, util::FxHashMap};

/// A marker indicating the entity can be collided with and should
/// have it's hash grid status computed.
#[derive(Component)]
pub struct Collidable;

/// A marker indicating a collider of a specified type.
///
/// Entities with a given `T` will have a `Coliding<T>` component
/// present when they are overlapping with any entities that have a `CollidesWith<T>`.
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
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl<T> Default for Collider<T>
where
    T: Send + Sync + 'static,
{
    fn default() -> Self {
        Self {
            phantom: PhantomData::<T>,
        }
    }
}

/// A marker indicating the entity can collide with colliders of the specified type.
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
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl<T> Default for CollidesWith<T>
where
    T: Send + Sync + 'static,
{
    fn default() -> Self {
        Self {
            phantom: PhantomData::<T>,
        }
    }
}

/// The `HashGrid` resource defines the coordinate bucket size to group entities into for
/// collision checking. This should be a few times the size of the largest entity.
pub struct HashGrid {
    pub size: i32,
}

/// A component for indicating the entities current hash grid cell.
///
/// This is updated in the `pre_update` phase of each frame, thus its value will be based
/// on where an entity was at the start of the frame.
#[derive(Component)]
struct HashMarker((i32, i32));

impl HashMarker {
    /// Returns a list of hash grid cell identifiers with the current cell as well as all immediately surrounding cells.
    pub const fn get_with_neighbors(&self, grid_size: i32) -> [(i32, i32); 9] {
        [
            self.0,
            (self.0 .0, self.0 .1 + grid_size),
            (self.0 .0, self.0 .1 - grid_size),
            (self.0 .0 + grid_size, self.0 .1),
            (self.0 .0 - grid_size, self.0 .1),
            (self.0 .0 + grid_size, self.0 .1 - grid_size),
            (self.0 .0 + grid_size, self.0 .1 + grid_size),
            (self.0 .0 - grid_size, self.0 .1 - grid_size),
            (self.0 .0 - grid_size, self.0 .1 + grid_size),
        ]
    }
}

/// Run before checking collisions, update the current hash marker for each entity whose position has changed.
#[allow(clippy::type_complexity)]
fn update_hash_marker(
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

/// For any entity that is [`Collidable`] and has a renderable position, add a [`HashMarker`].
///
/// The side effect of this is that it may take one frame after spawning before an instance can be collided with.
#[allow(clippy::type_complexity)]
fn insert_hash_marker(
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

/// A component present when the current entity is overlapping with a [`Collider<T>`] of the same `T`.
///
/// The contained [`Vec`] is a list of the Entities which were overlapping at the start of the frame.
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Colliding<T>(pub Vec<Entity>, PhantomData<T>);

type HashGridCoord = (i32, i32);

fn collisions<T>(
    collider_query: Query<(Entity, &Instance2D, &HashMarker), With<Collider<T>>>,
    collide_with_query: Query<(Entity, &Instance2D, &HashMarker), With<CollidesWith<T>>>,
    hash_grid: Res<HashGrid>,
    mut commands: Commands,
) where
    T: Send + Sync + 'static,
{
    let mut m: FxHashMap<HashGridCoord, Vec<(Entity, Instance2D)>> =
        HashMap::with_capacity_and_hasher(1000, BuildHasherDefault::<FxHasher>::default());
    for (entity, inst, hash_marker) in &collide_with_query {
        m.entry(hash_marker.0)
            .and_modify(|v| v.push((entity, *inst)))
            .or_insert_with(|| Vec::from([(entity, *inst)]));
    }

    for (entity, inst, hash_marker) in &collider_query {
        let mut collisions = Vec::new();
        for marker in hash_marker.get_with_neighbors(hash_grid.size) {
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

#[must_use]
pub fn base_collision_detection() -> SystemSet {
    SystemSet::new()
        .with_system(update_hash_marker)
        .label("collision_update")
        .with_system(insert_hash_marker)
}

#[must_use]
pub fn collision_system_set<T>() -> SystemSet
where
    T: Send + Sync + 'static,
{
    SystemSet::new()
        .with_system(collisions::<T>)
        .after("collision_update")
}

/// Given 2 instances, determine if they are overlapping.
///
/// This computes a bounding box for each instance that is `instance.scale.x` wide and `instance.scale.y` high.
/// It currently does not account for rotation, and assumes that the shape vertices are normalized to coordinates
/// between -1.0 and 1.0 on both axes.
#[allow(clippy::similar_names)]
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
    #[allow(clippy::cast_possible_truncation)]
    let mut res = (i as i32).abs() + incr / 2;
    res -= res % incr;
    if i < 0.0 {
        -res
    } else {
        res
    }
}

#[cfg(test)]
mod tests {
    use glam::{Vec2, Vec4};

    use crate::instance::Instance2D;

    use super::{overlapping, round_to_nearest, HashGridVec, HashMarker};

    #[test]
    fn test_round_to_nearest() {
        assert_eq!(round_to_nearest(250.0, 100), 300);
        assert_eq!(round_to_nearest(249.9, 100), 200);
        assert_eq!(round_to_nearest(-250.0, 100), -300);
        assert_eq!(round_to_nearest(-249.9, 100), -200);
    }

    #[test]
    fn test_hash_grid() {
        assert_eq!(Vec2::new(250.0, 0.0).current_hash_grid(100), (300, 0));
        assert_eq!(Vec2::new(-250.0, 0.0).current_hash_grid(100), (-300, 0));
        assert_eq!(Vec2::new(0.0, 250.0).current_hash_grid(100), (0, 300));
        assert_eq!(Vec2::new(0.0, -250.0).current_hash_grid(100), (0, -300));
    }

    #[test]
    fn test_overlapping() {
        let a = Instance2D {
            position: Vec2::new(-249.5, 500.0),
            rotation: 0.0,
            scale: Vec2::splat(35.0),
            color: Vec4::ZERO,
            shape: 0,
            outline: None,
        };

        let b = Instance2D {
            position: Vec2::new(-250.0, 500.0),
            rotation: 0.0,
            scale: Vec2::splat(50.0),
            color: Vec4::ZERO,
            shape: 0,
            outline: None,
        };

        let c = Instance2D {
            position: Vec2::new(-200.0, 500.0),
            rotation: 0.0,
            scale: Vec2::splat(10.0),
            color: Vec4::ZERO,
            shape: 0,
            outline: None,
        };

        assert!(overlapping(&a, &b));
        assert!(!overlapping(&a, &c));
    }

    #[test]
    fn test_neighbors() {
        assert_eq!(
            HashMarker((200, 0)).get_with_neighbors(100),
            [
                (200, 0),
                (200, 100),
                (200, -100),
                (300, 0),
                (100, 0),
                (300, -100),
                (300, 100),
                (100, -100),
                (100, 100)
            ]
        );
    }
}
