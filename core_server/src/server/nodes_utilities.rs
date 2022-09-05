use crate::prelude::*;
use pathfinding::prelude::bfs;
/*
/// Retrieves a number instance value
pub fn get_number_variable(instance_index: usize, variable: String, data: &mut RegionInstance) -> Option<f64> {
    if let Some(value) = data.scopes[instance_index].get_value::<f64>(&variable) {
        return Some(value.clone());
    }
    None
}

/// Retrieves a number instance value or 0
pub fn get_number_variable_or_zero(instance_index: usize, variable: String, data: &RegionInstance) -> f64 {
    if let Some(value) = data.scopes[instance_index].get_value::<f64>(&variable) {
        return value.clone();
    }
    0.0
}

/// Sets a number instance value
pub fn set_number_variable(instance_index: usize, variable: String, value: f64, data: &mut RegionInstance) {
    data.scopes[instance_index].set_value(&variable, value);
}

/// Retrieves a node value
pub fn get_node_value(id: (usize, usize, &str), data: &mut RegionInstance, behavior_type: BehaviorType) -> Option<(f64, f64, f64, f64, String)> {
    if behavior_type == BehaviorType::Regions {

        let behavior = &mut data.region_behavior[id.0];
        if let Some(node) = behavior.nodes.get_mut(&id.1) {
            if let Some(value) = node.values.get_mut(id.2) {
                return Some(value.clone());
            }
        } else
        if let Some(behavior) = data.behaviors.get_mut(&id.0) {
            if let Some(node) = behavior.nodes.get_mut(&id.1) {
                if let Some(value) = node.values.get_mut(id.2) {
                    return Some(value.clone());
                }
            }
        }
    } else
    if behavior_type == BehaviorType::Behaviors {
        if let Some(behavior) = data.behaviors.get_mut(&id.0) {
            if let Some(node) = behavior.nodes.get_mut(&id.1) {
                if let Some(value) = node.values.get_mut(id.2) {
                    return Some(value.clone());
                }
            }
        }
    } else
    if behavior_type == BehaviorType::Systems {
        if let Some(system) = data.systems.get_mut(&id.0) {
            if let Some(node) = system.nodes.get_mut(&id.1) {
                if let Some(value) = node.values.get_mut(id.2) {
                    return Some(value.clone());
                }
            }
        }
    } else
    if behavior_type == BehaviorType::GameLogic {
        let game = &mut data.game_data;
        if let Some(node) = game.nodes.get_mut(&id.1) {
            if let Some(value) = node.values.get_mut(id.2) {
                return Some(value.clone());
            }
        }
    }
    None
}

/// Computes the distance between two locations
pub fn compute_distance(p0: &(usize, isize, isize), p1: &(usize, isize, isize)) -> f64 {
    let dx = p0.1 - p1.1;
    let dy = p0.2 - p1.2;
    ((dx * dx + dy * dy) as f64).sqrt()
}

/// Returns the current position of the instance_index, takes into account an ongoing animation
pub fn get_instance_position(inst_index: usize, instances: &Vec<BehaviorInstance>) -> Option<(usize, isize, isize)> {
    if let Some(old_position) = instances[inst_index].old_position {
        return Some(old_position);
    }
    instances[inst_index].position
}

pub fn walk_towards(instance_index: usize, p: Option<(usize, isize, isize)>, dp: Option<(usize, isize, isize)>, exclude_dp: bool, data: &mut RegionInstance) -> BehaviorNodeConnector {

    // Cache the character positions
    let mut char_positions : Vec<(usize, isize, isize)> = vec![];

    if let Some(p) = p {
        for inst_index in 0..data.instances.len() {
            if inst_index != instance_index {
                // Only track if the state is normal
                if data.instances[inst_index].state == BehaviorInstanceState::Normal {
                    if let Some(pos) = data.instances[inst_index].position {
                        if p.0 == pos.0 {
                            if exclude_dp == false {
                                char_positions.push(pos);
                            } else {
                                // Exclude dp, otherwise the Close In tracking function does not find a route
                                if let Some(dp) = dp {
                                    if dp != pos {
                                        char_positions.push(pos);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if let Some(p) = p {

        let can_go = |x: isize, y: isize| -> bool {

            // Check tiles
            let tiles = data.get_tile_at((x, y));
            if tiles.is_empty() { return false; }
            for tile in tiles {
                if tile.usage == TileUsage::EnvBlocking || tile.usage == TileUsage::Water {
                    return false;
                }
            }

            // Check characters
            for char_p in &char_positions {
                if char_p.1 == x && char_p.2 == y {
                    return false;
                }
            }

            true
        };

        if let Some(dp) = dp {

            let result = bfs(&(p.1, p.2),
                                |&(x, y)| {
                                let mut v : Vec<(isize, isize)> = vec![];
                                if can_go(x + 1, y) { v.push((x + 1, y))};
                                if can_go(x, y + 1) { v.push((x, y + 1))};
                                if can_go(x - 1, y) { v.push((x - 1, y))};
                                if can_go(x, y - 1) { v.push((x, y - 1))};
                                v
                                },
                                |&p| p.0 == dp.1 && p.1 == dp.2);

            if let Some(result) = result {
                if result.len() > 1 {
                    data.instances[instance_index].old_position = data.instances[instance_index].position.clone();
                    data.instances[instance_index].position = Some((p.0, result[1].0, result[1].1));
                    return BehaviorNodeConnector::Right;
                } else
                if result.len() == 1 && dp.1 == result[0].0 && dp.2 == result[0].1 {
                    return BehaviorNodeConnector::Success;
                }
            }
        }
    }

    BehaviorNodeConnector::Fail
}*/