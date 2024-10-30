use crate::{
    component::{
        display::DisplayComponent,
        input::InputComponent,
        memory::{MemoryComponent, MemoryTranslationTable},
        schedulable::SchedulableComponent,
        Component, FromConfig,
    },
    input::EmulatedGamepad,
    rom::RomManager,
    runtime::{RenderingBackend, RenderingBackendState},
    task::{InitializeableTask, Task},
};
use downcast_rs::DowncastSync;
use num::rational::Ratio;
use sealed::sealed;
use std::{
    any::TypeId,
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub mod definitions;
pub mod executor;
pub mod initializer;

#[sealed]
trait MutexedComponent: DowncastSync {}
#[sealed]
impl<C: Component> MutexedComponent for Mutex<C> {}

#[derive(Default)]
pub struct QueryableComponents(HashMap<(TypeId, &'static str), Arc<dyn MutexedComponent>>);

impl QueryableComponents {
    pub fn query_component<C: Component>(&self, name: &'static str) -> Option<Arc<Mutex<C>>> {
        self.0
            .get(&(TypeId::of::<C>(), name))
            .cloned()
            .and_then(|component| component.into_any_arc().downcast::<Mutex<C>>().ok())
    }
}

// Intermediate state for the runtime to construct a emulation context out of it
pub struct Machine<R: RenderingBackend> {
    pub tasks: Vec<(Ratio<u32>, Box<dyn Task>)>,
    pub memory_translation_table: Arc<MemoryTranslationTable>,
    pub controllers: Vec<Arc<EmulatedGamepad>>,
    pub display_components: Vec<Arc<Mutex<dyn DisplayComponent<R>>>>,
}

impl<R: RenderingBackend> Machine<R> {
    pub fn build(
        rom_manager: Arc<RomManager>,
        rendering_state: &mut <R as RenderingBackend>::RuntimeState,
    ) -> MachineBuilder<R> {
        MachineBuilder {
            components: HashMap::new(),
            tasks: Vec::new(),
            rom_manager,
            memory_translation_table: MemoryTranslationTable::default(),
            queryable_components: QueryableComponents::default(),
            display_components: Vec::new(),
            controllers: Vec::new(),
            rendering_state,
        }
    }
}

pub struct MachineBuilder<'a, R: RenderingBackend> {
    /// Components
    components: HashMap<(TypeId, &'static str), Arc<Mutex<dyn Component>>>,
    /// Tasks wrapping scheduable components
    tasks: Vec<(Ratio<u32>, Box<dyn Task>)>,
    /// Memory translation table
    memory_translation_table: MemoryTranslationTable,
    /// Display components to be hooked with the runtime graphics backends
    display_components: Vec<Arc<Mutex<dyn DisplayComponent<R>>>>,
    /// Controllers
    controllers: Vec<Arc<EmulatedGamepad>>,
    /// Components stored in a downcastable way
    queryable_components: QueryableComponents,
    /// ROM manager
    rom_manager: Arc<RomManager>,
    /// Rendering runtime component for initializing display components
    rendering_state: &'a mut <R as RenderingBackend>::RuntimeState,
}

impl<'a, R: RenderingBackend> MachineBuilder<'a, R> {
    pub fn component<C: FromConfig>(
        self,
        name: &'static str,
        config: C::Config,
    ) -> ComponentBuilder<'a, R, C> {
        let component = C::from_config(self.rom_manager.clone(), config);

        ComponentBuilder {
            name,
            component: Arc::new(Mutex::new(component)),
            machine_builder: self,
        }
    }

    pub fn component_default<C: FromConfig>(self, name: &'static str) -> ComponentBuilder<'a, R, C>
    where
        C::Config: Default,
    {
        self.component(name, C::Config::default())
    }

    pub fn finalize_machine(self) -> Machine<R> {
        for component in self.components.values() {
            component
                .lock()
                .unwrap()
                .query_components(&self.queryable_components);
        }

        self.rendering_state
            .initialize_components(&self.display_components);

        Machine {
            tasks: self.tasks,
            memory_translation_table: Arc::new(self.memory_translation_table),
            controllers: self.controllers,
            display_components: self.display_components,
        }
    }
}

pub struct ComponentBuilder<'a, R: RenderingBackend, C: Component> {
    name: &'static str,
    component: Arc<Mutex<C>>,
    machine_builder: MachineBuilder<'a, R>,
}

impl<'a, R: RenderingBackend, C: Component> ComponentBuilder<'a, R, C> {
    pub fn finalize_component(self) -> MachineBuilder<'a, R> {
        let mut machine_builder = self.machine_builder;
        machine_builder
            .queryable_components
            .0
            .insert((TypeId::of::<C>(), self.name), self.component.clone());
        machine_builder
            .components
            .insert((TypeId::of::<C>(), self.name), self.component);
        machine_builder
    }
}

impl<'a, R: RenderingBackend, C: SchedulableComponent> ComponentBuilder<'a, R, C> {
    pub fn insert_schedule<T: InitializeableTask<C>>(
        mut self,
        config: T::Config,
    ) -> ComponentBuilder<'a, R, C> {
        let task = T::new(self.component.clone(), config);

        self.machine_builder
            .tasks
            .push((self.component.lock().unwrap().tick_rate(), Box::new(task)));

        self
    }

    pub fn insert_schedule_default<T: InitializeableTask<C>>(self) -> ComponentBuilder<'a, R, C>
    where
        T::Config: Default,
    {
        self.insert_schedule::<T>(T::Config::default())
    }
}

impl<'a, R: RenderingBackend, C: MemoryComponent> ComponentBuilder<'a, R, C> {
    pub fn with_memory_map(mut self) -> ComponentBuilder<'a, R, C> {
        self.machine_builder.memory_translation_table.insert(
            self.component.lock().unwrap().assigned_memory_range(),
            self.component.clone(),
        );

        self
    }
}

impl<'a, R: RenderingBackend, C: DisplayComponent<R>> ComponentBuilder<'a, R, C> {
    pub fn with_displayable(mut self) -> ComponentBuilder<'a, R, C> {
        self.machine_builder
            .display_components
            .push(self.component.clone());

        self
    }
}

impl<'a, R: RenderingBackend, C: InputComponent> ComponentBuilder<'a, R, C> {
    pub fn with_gamepad(mut self) -> ComponentBuilder<'a, R, C> {
        let assigned_inputs = self.component.lock().unwrap().registered_inputs();
        let controller = EmulatedGamepad::new(assigned_inputs);
        self.machine_builder.controllers.push(controller.clone());
        self.component.lock().unwrap().assign_controller(controller);
        self
    }
}
