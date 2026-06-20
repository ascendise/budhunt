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
            ) -> Vec<$crate::Components> {
                $script(entity, events)
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
        let mut updates: Vec<(usize, Vec<Components>)> = vec![];
        for entity in scripted_entities {
            let scripts = component!(&entity[Components::SCRIPTS], Components::Scripts);
            for script in scripts {
                let updated_components = script.run(&entity, events);
                updates.push((entity.id, updated_components));
            }
        }
        for (e, updated_components) in updates {
            entities.update_entity_batch(e, updated_components);
        }
    }
}

pub trait Script<T: Component = Components> {
    fn run(&self, entity: &Entity<'_, T>, events: &Events) -> Vec<T>;
}
