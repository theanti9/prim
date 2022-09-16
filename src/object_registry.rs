use crate::{instance::Instance2D, state::State, time::Time};

pub trait Component: Send + Sync + std::any::Any {
    fn update(&mut self, time: &Time, state: &State);
    fn get_renderables(&self) -> &Vec<Instance2D>;
}

pub struct GameObject {
    id: u32,
    components: Vec<Box<dyn Component>>,
}

impl GameObject {
    #[inline(always)]
    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn update(&mut self, time: &Time, state: &State) {
        for component in &mut self.components {
            component.update(time, state);
        }
    }

    pub fn add_component<T>(&mut self, component: T)
    where
        T: Component + 'static,
    {
        self.components.push(Box::new(component))
    }

    pub fn get_all_renderables<'a>(&'a self) -> Vec<&Instance2D> {
        self.components
            .iter()
            .flat_map(|f| f.get_renderables())
            .collect::<Vec<_>>()
    }
}

pub struct ObjectRegistry {
    objects: Vec<GameObject>,
}

impl ObjectRegistry {
    pub fn new() -> Self {
        Self {
            objects: Vec::with_capacity(1000),
        }
    }

    pub fn spawn_object(&mut self) -> &mut GameObject {
        {
            self.objects.push(GameObject {
                id: self.objects.len() as u32,
                components: vec![],
            });
        }
        let len = self.objects.len();
        &mut self.objects[len - 1]
    }

    pub fn update(&mut self, time: &Time, state: &State) {
        for object in &mut self.objects {
            object.update(time, state);
        }
    }

    pub fn collect_renderables(&self) -> Vec<&Instance2D> {
        let mut insts = self
            .objects
            .iter()
            .flat_map(|v| v.get_all_renderables())
            .collect::<Vec<_>>();
        insts.sort_by(|a, b| a.shape.cmp(&b.shape));

        insts
    }
}
