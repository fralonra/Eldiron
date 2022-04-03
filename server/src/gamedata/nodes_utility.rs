use crate::gamedata::GameData;
use evalexpr::*;
use rand::prelude::*;

use super::behavior::BehaviorNodeType;

/// Retrieves a number instance value
pub fn get_number_variable(instance_index: usize, variable: String, data: &mut GameData) -> Option<f64> {
    if let Some(value) = data.instances[instance_index].values.get(&variable) {
        return Some(value.clone());
    }
    None
}

/// Sets a number instance value
pub fn set_number_variable(instance_index: usize, variable: String, value: f64, data: &mut GameData) {
    if let Some(v) = data.instances[instance_index].values.get_mut(&variable) {
        *v = value;
    }
}

/// Retrieves a node value
pub fn get_node_value(id: (usize, usize, &str), data: &mut GameData) -> Option<(f64, f64, f64, f64, String)> {
    if let Some(behavior) = data.behaviors.get_mut(&id.0) {
        if let Some(node) = behavior.data.nodes.get_mut(&id.1) {
            if let Some(value) = node.values.get_mut(id.2) {
                return Some(value.clone());
            }
        }
    }
    None
}

/// Sets a node value
pub fn set_node_value(id: (usize, usize, &str), data: &mut GameData, value: (f64, f64, f64, f64, String)) {
    if let Some(behavior) = data.behaviors.get_mut(&id.0) {
        if let Some(node) = behavior.data.nodes.get_mut(&id.1) {
            if let Some(v) = node.values.get_mut(id.2) {
                *v = value;
            }
        }
    }
}

/// Evaluates a node expression as a number
pub fn eval_expression_as_number(instance_index: usize, id: (usize, usize), data: &mut GameData, value_id: &str, default: f64) -> f64 {
    if let Some(behavior) = data.behaviors.get_mut(&id.0) {

        // Insert the variables
        let mut cont = HashMapContext::new();
        for (key, value) in &data.instances[instance_index].values {
            let t = format!("{} = {}", key, value);
            let _ = eval_empty_with_context_mut(t.as_str(), &mut cont);
        }

        // d1 - d20
        let mut rng = thread_rng();
        for d in (2..=20).step_by(2) {
            let random = rng.gen_range(1..=d);
            let t = format!("{} = {}", format!("d{}", d), random);
            let _ = eval_empty_with_context_mut(t.as_str(), &mut cont);
        }

        // Evaluate the expression as a number
        if let Some(node) = behavior.data.nodes.get_mut(&id.1) {
            let exp = eval_with_context(&node.values.get(value_id).unwrap().4, &cont);
            if exp.is_ok() {
                let rc = exp.unwrap().as_number();
                if rc.is_ok() {
                    return rc.unwrap();
                }
            }
            /*
            return match exp {
                Ok(v) => {
                    match v.as_number() {
                        Ok(fv) => {
                            Some(fv)
                        },
                        Err(e) => None
                    }
                },
                Err(e) => None
            }*/
        }
    }
    default
}

/// Evaluates a node expression as a variable value and assign it
pub fn eval_expression_as_variable(instance_index: usize, id: (usize, usize), data: &mut GameData, value_id: &str) {
    if let Some(behavior) = data.behaviors.get_mut(&id.0) {

        // Insert the variables
        let mut cont = HashMapContext::new();
        for (key, value) in &data.instances[instance_index].values {
            let t = format!("{} = {}", key, value);
            let _ = eval_empty_with_context_mut(t.as_str(), &mut cont);
        }

        // d1 - d20
        let mut rng = thread_rng();
        for d in (2..=20).step_by(2) {
            let random = rng.gen_range(1..=d);
            let t = format!("{} = {}", format!("d{}", d), random);
            let _ = eval_empty_with_context_mut(t.as_str(), &mut cont);
        }

        // Evaluate the expression as a number
        if let Some(node) = behavior.data.nodes.get_mut(&id.1) {
            let exp = eval_empty_with_context_mut(&node.values.get(value_id).unwrap().4, &mut cont);
            if exp.is_ok() {

                let mut key_to_change: Option<String> = None;
                let mut new_value : Option<f64> = None;

                for (key, value) in &data.instances[instance_index].values {
                    if let Some(v) = cont.get_value(key) {
                        //println!("here {}", v);
                        let n = v.as_number();
                        if n.is_ok() {
                            let nn = n.unwrap();
                            if nn != *value {
                                key_to_change = Some(key.clone());
                                new_value = Some(nn);
                                break;
                            }
                        }
                    }
                }

                if let Some(key) = key_to_change {
                    if let Some(value) = new_value {

                        data.instances[instance_index].values.insert(key.clone(), value);

                        // Insert the node id of the changed variable to the list
                        // Note: Only need todo when run in editor
                        for (_index, node) in &behavior.data.nodes {
                            if node.name == key && node.behavior_type == BehaviorNodeType::VariableNumber {
                                data.changed_variables.push((instance_index, behavior.data.id, node.id, value));
                                //println!("{:?}", (instance_index, behavior.data.id, node.id));
                            }
                        }
                    }
                }
            }
        }
    }
}