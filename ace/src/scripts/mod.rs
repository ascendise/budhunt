use crate::*;

#[cfg(test)]
mod tests;

#[macro_export]
macro_rules! script {
    ($script:expr) => {{
        #[derive(Clone)]
        pub struct QuickScript;
        impl $crate::Script for QuickScript {
            fn run(
                &self,
                entity: &$crate::Entity<'_, $crate::Components>,
                events: &$crate::Events,
                updates: &mut $crate::Update<$crate::Components>,
            ) {
                $script(entity, events);
            }
        }
        QuickScript
    }};
}

pub struct ScriptSystem;
impl ScriptSystem {}
impl System for ScriptSystem {
    fn run(&self, entities: &mut Entities, events: &Events) {
        let scripted_entities = entities.get_entities(Components::SCRIPTS);
        let mut updates = entities.update();
        for entity in scripted_entities {
            let scripts = component!(&entity[Components::SCRIPTS], Components::Scripts);
            for script in scripts {
                script.run(&entity, events, &mut updates);
            }
        }
        entities.commit(updates);
    }
}

pub trait Script<T: Component = Components> {
    fn run(&self, entity: &Entity<'_, T>, events: &Events, updates: &mut Update<T>);
}
