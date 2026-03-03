use crate::*;

#[cfg(test)]
mod tests;

pub struct ScriptSystem;
impl ScriptSystem {}
impl System for ScriptSystem {
    fn run(&self, entities: &mut Entities, inputs: &[Input]) {
        let scripted_entities = entities.get_entities(Component::SCRIPTS);
        let mut updates: Vec<(usize, Vec<Component>)> = vec![];
        for (e, entity) in scripted_entities {
            let scripts = entity.iter().find(|e| matches!(e, Component::Scripts(_)));
            if let Some(Component::Scripts(scripts)) = scripts {
                for script in scripts {
                    let updated_components = script.run(&entity, inputs);
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

pub trait Script<T: TypeId = Component> {
    fn run(&self, entity: &[&T], inputs: &[Input]) -> Vec<T>;
}
