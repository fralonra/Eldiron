use crate::prelude::*;
use rhai::{ Dynamic };

use regex::bytes::Regex;

#[derive(Debug, Clone)]
struct InstanceVariables {
    pub numbers: HashMap<String, f64>
}

impl InstanceVariables {
    fn get_number(&mut self, index: String) -> f64 {
        if let Some(value) = self.numbers.get(&index) {
            return *value;
        }
        0.0
    }
    fn set_number(&mut self, index: String, value: f64) {
        if self.numbers.contains_key(&index) {
            self.numbers.insert(index, value);
        }
    }

    fn new() -> Self {
        Self { numbers: HashMap::new() }
    }
}

/// Adds the given target variables to the scope
pub fn add_target_to_scope(instance_index: usize, data: &mut RegionInstance) {
    if let Some(_target_index) = data.instances[instance_index].target_instance_index {
        data.engine.register_type::<InstanceVariables>()
            .register_fn("new_instance", InstanceVariables::new)
            .register_indexer_get(InstanceVariables::get_number)
            .register_indexer_set(InstanceVariables::set_number);


        //let original_target = data.scopes[target_index].clone();
        let target = InstanceVariables::new();

        /* TODO
        if let Some(behavior) = data.get_mut_behavior(data.instances[target_index].behavior_id, BehaviorType::Behaviors) {
            for (_index, node) in &behavior.nodes {
                if node.behavior_type == BehaviorNodeType::VariableNumber {

                    if let Some(v) = original_target.get_value::<f64>(node.name.as_str()) {
                        target.numbers.insert(node.name.clone(), v);
                    }
                }
            }
        }*/

        data.scopes[instance_index].set_value("target", target);
    }
}

/// Read out the target variables and if changed apply them
pub fn apply_scope_to_target(_instance_index: usize, _data: &mut RegionInstance) {
    /*
    if let Some(target_index) = data.instances[instance_index].target_instance_index {
        if let Some(target) = data.scopes[instance_index].get_value::<InstanceVariables>("target") {
            if let Some(behavior) = data.behaviors.get_mut(&data.instances[target_index].behavior_id) {
                for (_index, node) in &behavior.nodes {
                    if node.behavior_type == BehaviorNodeType::VariableNumber {

                        let o = data.scopes[target_index].get_value::<f64>(node.name.as_str());
                        let n = target.numbers.get(&node.name);

                        if n.is_some() && o.is_some() && o.unwrap() != *n.unwrap() {
                            let value = n.unwrap().clone();
                            data.scopes[target_index].set_value(node.name.clone(), value);
                        }
                    }
                }
            }
        }
    }*/
}

/// Replace the target strings, only called once before compilation for each script
pub fn replace_target_variables(input: String) -> String {
    let output = input.clone();
    if input.contains("${Target}") {
        if let Some(re) = Regex::new(r"\$\{Target\}.(?P<v>\w*)").ok() {
            let t = re.replace_all(output.as_bytes(), "target[\"$v\"]".as_bytes());
            if let Some(tt) = std::str::from_utf8(t.as_ref()).ok() {
                return tt.to_string();
            }
        }
    }
    output
}

/// Evaluates a boolean expression in the given instance.
pub fn eval_bool_expression_instance(instance_index: usize, id: (BehaviorType, Uuid, Uuid, String), data: &mut RegionInstance) -> Option<bool> {
    add_target_to_scope(instance_index, data);

    if let Some(ast) = data.ast.get(&id) {
        let r = data.engine.eval_ast_with_scope(&mut  data.scopes[instance_index], ast);
        if r.is_ok() {
            return Some(r.unwrap());
        } else {
            println!("{:?}", r);
        }
    } else {
        if let Some(value) = get_node_value((id.1, id.2, &id.3), data, id.0) {
            if let Some(code) = value.to_string() {
                let script = replace_target_variables(code);
                if let Some(ast) = data.engine.compile_expression_with_scope(&mut  data.scopes[instance_index], script.as_str()).ok() {
                    let r = data.engine.eval_ast_with_scope(&mut  data.scopes[instance_index], &ast);
                    if r.is_ok() {
                        data.ast.insert(id.clone(), ast);
                        return Some(r.unwrap());
                    } else {
                        println!("{:?}", r);
                    }
                }
            }
        }
    }

    None
}

/// Evaluates a numerical expression in the given instance.
pub fn eval_number_expression_instance(instance_index: usize, id: (BehaviorType, Uuid, Uuid, String), data: &mut RegionInstance) -> Option<f32> {
    add_target_to_scope(instance_index, data);

    if let Some(ast) = data.ast.get(&id) {
        let r = data.engine.eval_ast_with_scope::<Dynamic>(&mut  data.scopes[instance_index], ast);
        if r.is_ok() {
            let nn = r.unwrap();
            if let Some(n) = nn.as_float().ok() {
                return Some(n);
            }
            if let Some(n) = nn.as_int().ok() {
                return Some(n as f32);
            }
        } else {
            println!("{:?}", r);
        }
    } else {
        if let Some(value) = get_node_value((id.1, id.2, &id.3), data, id.0) {
            if let Some(code) = value.to_string() {
                let script = replace_target_variables(code);
                if let Some(ast) = data.engine.compile_expression_with_scope(&mut  data.scopes[instance_index], script.as_str()).ok() {
                    let r = data.engine.eval_ast_with_scope::<Dynamic>(&mut  data.scopes[instance_index], &ast);
                    if r.is_ok() {
                        data.ast.insert(id.clone(), ast);
                        let nn = r.unwrap();
                        if let Some(n) = nn.as_float().ok() {
                            return Some(n);
                        }
                        if let Some(n) = nn.as_int().ok() {
                            return Some(n as f32);
                        }
                    } else {
                        println!("{:?}", r);
                    }
                }
            }
        }
    }
    None
}

/// Evaluates a dynamic script in the given instance.
pub fn eval_dynamic_script_instance(instance_index: usize, id: (BehaviorType, Uuid, Uuid, String), data: &mut RegionInstance) -> bool {

    add_target_to_scope(instance_index, data);

    if let Some(ast) = data.ast.get(&id) {
        let r = data.engine.eval_ast_with_scope::<Dynamic>(&mut  data.scopes[instance_index], ast);
        if r.is_ok() {
            apply_scope_to_target(instance_index, data);
            return true
        } else {
            println!("{:?}", r);
        }
    } else {
        if let Some(value) = get_node_value((id.1, id.2, &id.3), data, id.0) {
            if let Some(code) = value.to_string() {
                let script = replace_target_variables(code);
                if let Some(ast) = data.engine.compile_with_scope(&mut  data.scopes[instance_index], script.as_str()).ok() {
                    let r = data.engine.eval_ast_with_scope::<Dynamic>(&mut  data.scopes[instance_index], &ast);
                    if r.is_ok() {
                        data.ast.insert(id.clone(), ast);
                        apply_scope_to_target(instance_index, data);
                        return true
                    } else
                    if let Some(err) = r.err() {
                        // data.instances[instance_index].messages.push(MessageData {
                        //     message_type        : MessageType::Error,
                        //     message             : err.to_string(),
                        //     from                : "Script".to_string()
                        // });
                        data.script_errors.push(
                            ((id.1, id.2, id.3), (err.to_string(), None))
                        );
                    }
                }
            }
        }
    }

    false
}

/// Evaluates a dynamic script in the given instance.
pub fn eval_dynamic_script_instance_for_game_player_scope(_instance_index: usize, id: (BehaviorType, Uuid, Uuid, String), data: &mut RegionInstance, custom_scope: usize) -> bool {

    if let Some(ast) = data.ast.get(&id) {
        if let Some(custom_scope) = data.game_player_scopes.get_mut(&custom_scope) {

            let r = data.engine.eval_ast_with_scope::<Dynamic>(custom_scope, ast);
            if r.is_ok() {
                return true
            } else {
                println!("{:?}", r);
            }
        }
    } else {
        if let Some(value) = get_node_value((id.1, id.2, &id.3), data, id.0) {
            if let Some(script) = value.to_string() {
                if let Some(ast) = data.engine.compile_with_scope(data.game_player_scopes.get_mut(&custom_scope).unwrap(), script.as_str()).ok() {
                    let r = data.engine.eval_ast_with_scope::<Dynamic>(data.game_player_scopes.get_mut(&custom_scope).unwrap(), &ast);
                    if r.is_ok() {
                        data.ast.insert(id.clone(), ast);
                        return true
                    } else {
                        println!("{:?}", r);
                    }
                }
            }
        }
    }

    false
}
