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
                entity: &[&$crate::Components],
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
        for (e, entity) in scripted_entities {
            let scripts = entity.iter().find(|e| matches!(e, Components::Scripts(_)));
            if let Some(Components::Scripts(scripts)) = scripts {
                for script in scripts {
                    let updated_components = script.run(&entity, events);
                    updates.push((e, updated_components));
                }
            }
        }
        for (e, update) in updates {
            for component in update {
                entities.update_entity(e, component);
            }
        }
    }
}

pub trait Script<T: Component = Components> {
    fn run(&self, entity: &[&T], events: &Events) -> Vec<T>;
}
