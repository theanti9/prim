use glam::{Vec2, Vec4};
use libprim::{
    components::movement::MovementController, instance::Instance2D, object_registry::Component,
    run, state::State, time::Time,
};
use rand::{thread_rng, Rng};

const NUM_INSTANCES_PER_ROW: u32 = 100;
pub struct Spinner {
    instances: Vec<Instance2D>,
    multiplier: f32,
}

impl Spinner {
    pub fn new(position: Vec2) -> Self {
        let mut rng = thread_rng();
        Self {
            instances: vec![Instance2D {
                position,
                rotation: 0.0,
                scale: Vec2::splat(35.0),
                color: Vec4::new(1.0, 0.5, 0.2, 1.0),
                shape: 0,
            }],
            multiplier: rng.gen_range(0.2..2.0),
        }
    }
}
impl Component for Spinner {
    fn update(&mut self, time: &Time, _state: &State) {
        self.instances[0].rotation += self.multiplier * time.delta_seconds();
    }

    fn get_renderables(&self) -> &Vec<Instance2D> {
        &self.instances
    }
}

fn spinner_test() {
    pollster::block_on(run(|state| {
        state.spawn(|obj| {
            let triangle = state.get_shape_id("Triangle").unwrap();
            let line = state.get_shape_id("Line").unwrap();

            let spinners = (0..NUM_INSTANCES_PER_ROW)
                .flat_map(|y| {
                    (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                        let position = Vec2::new(
                            (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0) * 40.0,
                            (y as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0) * 40.0,
                        );
                        let mut spinner = Spinner::new(position);
                        spinner.instances[0].scale = Vec2::splat(35.0);
                        spinner.instances[0].color = Vec4::new(
                            position.x / 50.0 / NUM_INSTANCES_PER_ROW as f32,
                            position.y / 50.0 / NUM_INSTANCES_PER_ROW as f32,
                            0.2,
                            1.0,
                        );
                        spinner.instances[0].shape = if (x + y) % 2 == 0 { triangle } else { line };
                        spinner
                    })
                })
                .collect::<Vec<_>>();

            for spinner in spinners {
                obj.add_component(spinner);
            }
        });
    }));
}

pub struct Player {
    instances: Vec<Instance2D>,
    movement_controller: MovementController,
}

impl Player {
    pub fn new() -> Self {
        let mut inst = Instance2D::new();
        inst.shape = 1;
        inst.scale = Vec2::splat(150.0);
        inst.color = Vec4::ONE;
        Self {
            instances: Vec::from([inst]),
            movement_controller: MovementController::new(500.0, Vec2::ZERO),
        }
    }
}

impl Component for Player {
    fn update(&mut self, time: &Time, state: &State) {
        self.movement_controller.update(time, state);
        self.instances[0].position = self.movement_controller.position;
    }

    fn get_renderables(&self) -> &Vec<Instance2D> {
        &self.instances
    }
}

fn movement_test() {
    pollster::block_on(run(|state| {
        state.spawn(|obj| {
            obj.add_component(Player::new());
        })
    }));
}

fn main() {
    //spinner_test();
    movement_test();
}
