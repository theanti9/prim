use bevy_ecs::system::{Commands, RemovedComponents, ResMut};
use glam::{Vec2, Vec4};
use libprim::{
    animation::{
        shape_cycle::{animation_system_set, Animation, AnimationBundle, TimePoint},
        tween::{tween_system_set, Tween, TweenState, Tweening, Tweens},
    },
    instance::{Instance2D, InstanceBundle, Outline},
    shape_registry::ShapeRegistry,
    window::PrimWindowOptions,
};

#[derive(Debug, Clone)]
struct ColorRotation {
    pub colors: [Vec4; 4],
    pub index: usize,
}

fn next_tween(
    mut color_rotation: ResMut<ColorRotation>,
    finished_tweens: RemovedComponents<Tweening>,
    mut commands: Commands,
) {
    for entity in finished_tweens.iter() {
        let prev_color_index = color_rotation.index;
        color_rotation.index += 1;
        if color_rotation.index == 4 {
            color_rotation.index = 0;
        }
        commands
            .entity(entity)
            .insert(Tweens(vec![Tween::tween_color(
                color_rotation.colors[prev_color_index],
                color_rotation.colors[color_rotation.index],
                1.0,
            )]))
            .insert(TweenState::default())
            .insert(Tweening);
    }
}

fn run_animation() {
    libprim::run(PrimWindowOptions::default(), |state| {
        let schedule = state.borrow_schedule();
        schedule.add_system_set_to_stage("update", animation_system_set());
        schedule.add_system_set_to_stage("update", tween_system_set());
        // Component removal detection state is cleared at the end of the frame, but systems which
        // rely on the detection need to run in a stage after the removal itself. So run this in post_update.
        schedule.add_system_to_stage("post_update", next_tween);
        let world = state.borrow_world();

        let shape_registry = world.get_resource::<ShapeRegistry>().unwrap();

        let line = shape_registry.get_id("Line").unwrap();
        let triangle = shape_registry.get_id("Triangle").unwrap();
        let square = shape_registry.get_id("Square").unwrap();

        let color_rotation = ColorRotation {
            colors: [
                Vec4::new(1.0, 1.0, 1.0, 1.0),
                Vec4::new(1.0, 0.0, 0.0, 1.0),
                Vec4::new(0.0, 1.0, 0.0, 1.0),
                Vec4::new(0.0, 0.0, 1.0, 1.0),
            ],
            index: 0,
        };

        world.insert_resource(color_rotation.clone());

        world
            .spawn()
            .insert_bundle(InstanceBundle::new(Instance2D {
                shape: line,
                scale: Vec2::splat(500.0),
                outline: Some(Outline {
                    scale: 55.0,
                    color: Vec4::ONE,
                }),
                ..Default::default()
            }))
            .insert_bundle(AnimationBundle::from_animation(Animation::new(
                vec![
                    TimePoint {
                        shape_id: line,
                        duration: 2.0,
                    },
                    TimePoint {
                        shape_id: triangle,
                        duration: 2.0,
                    },
                    TimePoint {
                        shape_id: square,
                        duration: 2.0,
                    },
                ],
                true,
                1.0,
            )))
            .insert(Tweens(vec![Tween::tween_color(
                color_rotation.colors[0],
                color_rotation.colors[1],
                1.0,
            )]))
            .insert(TweenState::default())
            .insert(Tweening);
    });
}

fn main() {
    run_animation();
}
