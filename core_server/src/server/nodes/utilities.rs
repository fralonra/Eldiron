extern crate ref_thread_local;
use crate::prelude::*;
use pathfinding::prelude::bfs;
use ref_thread_local::RefThreadLocal;

/// Returns an integer value for the given node.
pub fn get_node_value2(
    id: (Uuid, Uuid),
    value_name: &str,
    nodes: &mut FxHashMap<Uuid, GameBehaviorData>,
) -> Option<Value> {
    if let Some(item) = nodes.get_mut(&id.0) {
        if let Some(node) = item.nodes.get_mut(&id.1) {
            for (name, value) in &node.values {
                if *name == value_name {
                    return Some(value.clone());
                }
            }
        }
    }
    None
}

/// Returns an integer value for the given node.
pub fn get_node_integer(
    id: (Uuid, Uuid),
    value_name: &str,
    nodes: &mut FxHashMap<Uuid, GameBehaviorData>,
) -> Option<i32> {
    if let Some(item) = nodes.get_mut(&id.0) {
        if let Some(node) = item.nodes.get_mut(&id.1) {
            for (name, value) in &node.values {
                if *name == value_name {
                    if let Some(int) = value.to_integer() {
                        return Some(int);
                    }
                    break;
                }
            }
        }
    }
    None
}

/// Returns an integer value for the given node.
pub fn get_node_string(
    id: (Uuid, Uuid),
    value_name: &str,
    nodes: &mut FxHashMap<Uuid, GameBehaviorData>,
) -> Option<String> {
    if let Some(item) = nodes.get_mut(&id.0) {
        if let Some(node) = item.nodes.get_mut(&id.1) {
            for (name, value) in &node.values {
                if *name == value_name {
                    if let Some(v) = value.to_string() {
                        return Some(v);
                    }
                    break;
                }
            }
        }
    }
    None
}

/*
/// Retrieves a number instance value
pub fn get_number_variable(instance_index: usize, variable: String, data: &mut RegionInstance) -> Option<f32> {
    if let Some(value) = data.scopes[instance_index].get_value::<f32>(&variable) {
        return Some(value);
    }
    None
}

/// Retrieves a number instance value or 0
pub fn get_number_variable_or_zero(instance_index: usize, variable: String, data: &RegionInstance) -> f32 {
    if let Some(value) = data.scopes[instance_index].get_value::<f32>(&variable) {
        return value;
    }
    0.0
}

/// Retrieves an i32 variable
pub fn get_i32_variable(instance_index: usize, variable: String, data: &mut RegionInstance) -> Option<i32> {
    if let Some(value) = data.scopes[instance_index].get_value::<i32>(&variable) {
        return Some(value);
    }
    if let Some(value) = data.scopes[instance_index].get_value::<f32>(&variable) {
        return Some(value as i32);
    }
    None
}

/// Retrieves an i32 variable
pub fn get_f32_variable(instance_index: usize, variable: String, data: &mut RegionInstance) -> Option<f32> {
    if let Some(value) = data.scopes[instance_index].get_value::<f32>(&variable) {
        return Some(value);
    }
    if let Some(value) = data.scopes[instance_index].get_value::<i32>(&variable) {
        return Some(value as f32);
    }
    None
}

/// Sets a number instance value
pub fn set_number_variable(instance_index: usize, variable: String, value: f32, data: &mut RegionInstance) {
    data.scopes[instance_index].set_value(&variable, value);
}*/

/// Retrieves a node value
pub fn get_node_value(
    id: (Uuid, Uuid, &str),
    data: &mut RegionInstance,
    behavior_type: BehaviorType,
) -> Option<Value> {
    if behavior_type == BehaviorType::Regions {
        for behavior in &data.region_behavior {
            //if behavior.id == id.0 {
            if let Some(node) = behavior.nodes.get(&id.1) {
                if let Some(value) = node.values.get(id.2) {
                    return Some(value.clone());
                }
            }
            //}
        }
    } else if behavior_type == BehaviorType::Behaviors {
        if let Some(behavior) = data.behaviors.get_mut(&id.0) {
            if let Some(node) = behavior.nodes.get_mut(&id.1) {
                if let Some(value) = node.values.get_mut(id.2) {
                    return Some(value.clone());
                }
            }
        }
    } else if behavior_type == BehaviorType::Systems {
        if let Some(system) = data.systems.get_mut(&id.0) {
            if let Some(node) = system.nodes.get_mut(&id.1) {
                if let Some(value) = node.values.get_mut(id.2) {
                    return Some(value.clone());
                }
            }
        }
    } else if behavior_type == BehaviorType::Items {
        if let Some(item) = data.items.get_mut(&id.0) {
            if let Some(node) = item.nodes.get_mut(&id.1) {
                if let Some(value) = node.values.get_mut(id.2) {
                    return Some(value.clone());
                }
            }
        }
    } else if behavior_type == BehaviorType::Spells {
        if let Some(item) = data.spells.get_mut(&id.0) {
            if let Some(node) = item.nodes.get_mut(&id.1) {
                if let Some(value) = node.values.get_mut(id.2) {
                    return Some(value.clone());
                }
            }
        }
    } else if behavior_type == BehaviorType::GameLogic {
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
pub fn compute_distance(p0: &Position, p1: &Position) -> i32 {
    let dx = p0.x - p1.x;
    let dy = p0.y - p1.y;
    ((dx * dx + dy * dy) as f32).sqrt().floor() as i32
}

/// Returns the current position of the instance_index, takes into account an ongoing animation
pub fn get_instance_position(
    inst_index: usize,
    instances: &Vec<BehaviorInstance>,
) -> Option<Position> {
    if let Some(old_position) = &instances[inst_index].old_position {
        return Some(old_position.clone());
    }
    instances[inst_index].position.clone()
}

/// Walk towards a destination position
pub fn walk_towards(
    p: Option<Position>,
    dp: Option<Position>,
    exclude_dp: bool,
    data: &mut RegionData,
) -> BehaviorNodeConnector {
    // Cache the character positions
    let mut char_positions: Vec<Position> = vec![];

    if let Some(p) = &p {
        for inst_index in 0..data.character_instances.len() {
            if inst_index != data.curr_index {
                // Only track if the state is normal
                if data.character_instances[inst_index].state == BehaviorInstanceState::Normal {
                    if let Some(pos) = &data.character_instances[inst_index].position {
                        if p.region == pos.region {
                            if exclude_dp == false {
                                char_positions.push(pos.clone());
                            } else {
                                // Exclude dp, otherwise the Close In tracking function does not find a route
                                if let Some(dp) = &dp {
                                    if *dp != *pos {
                                        char_positions.push(pos.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if let Some(p) = &p {
        let can_go = |x: isize, y: isize| -> bool {
            // Check tiles
            let tiles = data.get_tile_at((x, y));
            if tiles.is_empty() {
                return false;
            }
            for tile in tiles {
                if tile.usage == TileUsage::EnvBlocking || tile.usage == TileUsage::Water {
                    return false;
                }
            }

            // Check characters
            for char_p in &char_positions {
                if char_p.x == x && char_p.y == y {
                    return false;
                }
            }

            // Check items
            if let Some(items) = data.loot.get(&(x, y)) {
                for item in items {
                    if item.state_blocking {
                        if let Some(state) = &item.state {
                            if state.state {
                                return false;
                            }
                        }
                    }
                }
            }

            true
        };

        if let Some(dp) = dp {
            let result = bfs(
                &(p.x, p.y),
                |&(x, y)| {
                    let mut v: Vec<(isize, isize)> = vec![];
                    if can_go(x + 1, y) {
                        v.push((x + 1, y))
                    };
                    if can_go(x, y + 1) {
                        v.push((x, y + 1))
                    };
                    if can_go(x - 1, y) {
                        v.push((x - 1, y))
                    };
                    if can_go(x, y - 1) {
                        v.push((x, y - 1))
                    };
                    v
                },
                |&p| p.0 == dp.x && p.1 == dp.y,
            );

            if let Some(result) = result {
                if result.len() > 1 {
                    if data.pixel_based_movement == true {
                        data.character_instances[data.curr_index].old_position =
                            data.character_instances[data.curr_index].position.clone();
                    }
                    data.character_instances[data.curr_index].position =
                        Some(Position::new(p.region, result[1].0, result[1].1));
                    return BehaviorNodeConnector::Right;
                } else if result.len() == 1 && dp.x == result[0].0 && dp.y == result[0].1 {
                    return BehaviorNodeConnector::Success;
                }
            }
        }
    }

    BehaviorNodeConnector::Fail
}

/// Executes the given action in the given direction, checking for areas, loot items and NPCs
pub fn execute_targeted_action(
    action_name: String,
    dp: Option<Position>,
    nodes: &mut FxHashMap<Uuid, GameBehaviorData>,
) -> BehaviorNodeConnector {
    // Find areas which contains the destination position and check if it has a fitting action node

    if let Some(dp) = &dp {
        let mut area_to_execute: Vec<(Uuid, usize, Uuid)> = vec![];

        // Check areas
        {
            let data: &mut RegionData = &mut REGION_DATA.borrow_mut()[*CURR_INST.borrow()];
            let mut ids: Vec<(Uuid, usize, Uuid)> = vec![];

            for (index, area) in data.region_data.areas.iter().enumerate() {
                for p in &area.area {
                    if p.0 == dp.x && p.1 == dp.y {
                        if let Some(behavior) = data.region_area_behavior.get(index) {
                            for (id, node) in &behavior.nodes {
                                if node.behavior_type == BehaviorNodeType::ActionArea {
                                    ids.push((area.behavior, index, *id));
                                }
                            }
                        }
                    }
                }
            }

            /// Returns a string value for the given node.
            fn get_node_string(
                id: Uuid,
                value_name: &str,
                nodes: &mut FxHashMap<Uuid, BehaviorNode>,
            ) -> Option<String> {
                if let Some(node) = nodes.get_mut(&id) {
                    for (name, value) in &node.values {
                        if *name == value_name {
                            if let Some(v) = value.to_string() {
                                return Some(v);
                            }
                            break;
                        }
                    }
                }
                None
            }

            for id in ids {
                let nodes: &mut HashMap<
                    Uuid,
                    BehaviorNode,
                    std::hash::BuildHasherDefault<rustc_hash::FxHasher>,
                > = &mut data.region_area_behavior[id.1].nodes;
                if let Some(name) = get_node_string(id.2, "action", nodes) {
                    if name == action_name {
                        data.curr_action_character_index = Some(data.curr_index);
                        area_to_execute.push(id);
                    }
                }
            }
        }

        // Need to execute an area node ?
        for id in area_to_execute {
            execute_area_node(id.0, id.1, id.2);
            let data: &mut RegionData = &mut REGION_DATA.borrow_mut()[*CURR_INST.borrow()];
            data.curr_action_character_index = None;
            return BehaviorNodeConnector::Success;
        }

        // Check loot items
        let mut loot = vec![];

        {
            let data = &REGION_DATA.borrow()[*CURR_INST.borrow()];
            if let Some(l) = data.loot.get(&(dp.x, dp.y)) {
                loot = l.clone();
            }
        }

        for index in 0..loot.len() {
            if loot[index].state.is_none() {
                // Check if we have to create the item state
                loot[index].state = check_and_create_item_state(
                    loot[index].id,
                    loot[index].exectute_on_startup.clone(),
                );
            }

            let mut to_execute = vec![];
            let mut item_nodes = ITEMS.borrow_mut();

            if let Some(behavior) = item_nodes.get(&loot[index].id) {
                for (id, node) in &behavior.nodes {
                    if node.behavior_type == BehaviorNodeType::BehaviorTree {
                        if node.name == action_name {
                            to_execute.push((behavior.id, *id));
                        }
                    }
                }
            }

            for (behavior_id, node_id) in to_execute {
                if let Some(state) = &loot[index].state {
                    *STATE.borrow_mut() = state.clone();
                    execute_node(behavior_id, node_id, &mut item_nodes);
                    loot[index].state = Some(STATE.borrow().clone());
                    let data = &mut REGION_DATA.borrow_mut()[*CURR_INST.borrow()];
                    data.loot.insert((dp.x, dp.y), loot);
                    return BehaviorNodeConnector::Success;
                } else {
                    execute_node(behavior_id, node_id, &mut item_nodes);
                    return BehaviorNodeConnector::Success;
                }
            }
        }

        // Check for characters at the dp

        let mut to_execute = vec![];

        {
            let data: &mut RegionData = &mut REGION_DATA.borrow_mut()[*CURR_INST.borrow()];
            for inst_index in 0..data.character_instances.len() {
                if inst_index != data.curr_index {
                    // Only track if the state is normal
                    if data.character_instances[inst_index].state == BehaviorInstanceState::Normal {
                        if let Some(pos) = &data.character_instances[inst_index].position {
                            if *dp == *pos {
                                if let Some(behavior) =
                                    nodes.get(&data.character_instances[inst_index].behavior_id)
                                {
                                    for (id, node) in &behavior.nodes {
                                        if node.behavior_type == BehaviorNodeType::BehaviorTree {
                                            if node.name == action_name.clone() + " (P)" {
                                                // Install the communication partner as the target for the player
                                                data.character_instances[data.curr_index]
                                                    .target_instance_index = Some(inst_index);
                                                to_execute.push((inst_index, behavior.id, *id));
                                            }
                                        }
                                    }
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }

        for (_inst_index, behavior_id, node_id) in to_execute {
            execute_node(behavior_id, node_id, nodes);
            return BehaviorNodeConnector::Success;
        }
    }
    BehaviorNodeConnector::Fail
}

/// Drops the communication between a player and an NPC
pub fn drop_communication(instance_index: usize, npc_index: usize, data: &mut RegionData) {
    // Drop Communication for the player

    data.character_instances[instance_index].multi_choice_answer = None;
    data.character_instances[instance_index].communication = vec![];
    data.character_instances[instance_index].multi_choice_data = vec![];

    // Drop comm for the NPC

    let mut com_to_drop: Option<usize> = None;
    for c_index in 0..data.character_instances[npc_index].communication.len() {
        if data.character_instances[npc_index].communication[c_index].player_index == instance_index
        {
            // Drop this communication for the NPC
            com_to_drop = Some(c_index);
            break;
        }
    }

    if let Some(index) = com_to_drop {
        data.character_instances[npc_index]
            .communication
            .remove(index);
    }
}

/// Check if we have to create the state for the given item
pub fn check_and_create_item_state(
    item_behavior_id: Uuid,
    execute_on_startup: Option<String>,
) -> Option<State> {
    let mut states_to_execute = vec![];
    let mut item_nodes = ITEMS.borrow_mut();

    if let Some(behavior) = item_nodes.get(&item_behavior_id) {
        let mut sink: Option<PropertySink> = None;

        // Get the default tile for the item
        for (_index, node) in &behavior.nodes {
            if node.behavior_type == BehaviorNodeType::BehaviorType {
                if let Some(value) = node.values.get(&"settings".to_string()) {
                    if let Some(str) = value.to_string() {
                        let mut s = PropertySink::new();
                        s.load_from_string(str.clone());
                        sink = Some(s);
                    }
                }
            }
        }

        // Add state ?

        if let Some(sink) = sink {
            if let Some(state) = sink.get("state") {
                if let Some(value) = state.as_bool() {
                    if value == true {
                        // Pass One, add the startup tree
                        for (node_id, node) in &behavior.nodes {
                            if node.behavior_type == BehaviorNodeType::BehaviorTree {
                                for (value_name, value) in &node.values {
                                    if *value_name == "execute".to_string() {
                                        if let Some(v) = value.to_integer() {
                                            if v == 1 {
                                                // Startup only tree
                                                states_to_execute.push((behavior.id, *node_id));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        // Pass two, add the execute_on_startup tree
                        for (node_id, node) in &behavior.nodes {
                            if Some(node.name.clone()) == execute_on_startup {
                                states_to_execute.push((behavior.id, *node_id));
                            }
                        }
                    }
                }
            }
        }
    }

    // Create the state
    if states_to_execute.is_empty() == false {
        *STATE.borrow_mut() = State::new();
        for (behavior_id, node_id) in states_to_execute {
            execute_node(behavior_id, node_id, &mut item_nodes);
        }
        return Some(STATE.borrow().clone());
    }

    None
}

/// Starts to wait for the given amount of ticks
pub fn wait_start(instance_index: usize, ticks: usize, id: (Uuid, Uuid), data: &mut RegionData) {
    data.character_instances[instance_index]
        .node_values
        .insert(id, Value::USize(ticks + *TICK_COUNT.borrow() as usize));
}

/// Waits for the given ticks to pass before returning true
pub fn wait_for(instance_index: usize, id: (Uuid, Uuid), data: &mut RegionData) -> bool {
    let mut rc = true;

    if let Some(value) = data.character_instances[instance_index]
        .node_values
        .get(&id)
    {
        match value {
            Value::USize(until) => {
                if *until >= *TICK_COUNT.borrow() as usize {
                    rc = false;
                } else {
                    data.character_instances[instance_index].node_values.clear();
                }
            }
            _ => {}
        }
    }
    rc
}

/// Returns the weapon distance for the given weapon slot
pub fn get_weapon_distance(slot: String, data: &mut RegionData) -> i32 {
    let mut weapon_distance = 1;

    let sheet: &mut Sheet = &mut data.sheets[data.curr_index];
    if let Some(weapon) = sheet.weapons.slots.get(&slot) {
        if weapon.weapon_distance > weapon_distance {
            weapon_distance = weapon.weapon_distance;
        }
    }

    weapon_distance
}

/// Returns the spell distance for the given spell name
pub fn get_spell_distance(name: String, data: &mut RegionData) -> i32 {
    let sheet: &mut Sheet = &mut data.sheets[data.curr_index];
    let spell = sheet.spells.get_spell(&name);
    let spell_distance = spell.distance;

    spell_distance
}

/// Returns the PropertySink for the given item id
pub fn get_item_sink(data: &RegionInstance, id: Uuid) -> Option<PropertySink> {
    for (uuid, item) in &data.items {
        if *uuid == id {
            for (_index, node) in &item.nodes {
                if node.behavior_type == BehaviorNodeType::BehaviorType {
                    if let Some(value) = node.values.get(&"settings".to_string()) {
                        if let Some(str) = value.to_string() {
                            let mut s = PropertySink::new();
                            s.load_from_string(str.clone());
                            return Some(s);
                        }
                    }
                }
            }
        }
    }
    None
}

/// Returns the skill name (if any) for the given item id
pub fn get_item_skill_tree(data: &RegionInstance, id: Uuid) -> Option<String> {
    for (uuid, item) in &data.items {
        if *uuid == id {
            for (_index, node) in &item.nodes {
                if node.behavior_type == BehaviorNodeType::SkillTree {
                    return Some(node.name.clone());
                }
            }
        }
    }
    None
}

/// Returns the script id for the given skill name and level
pub fn get_skill_script_id(
    data: &RegionInstance,
    item_name: String,
    _skill_name: String,
    skill_level: i32,
) -> Option<(BehaviorType, Uuid, Uuid, String)> {
    for (_uuid, behavior) in &data.items {
        if behavior.name == item_name {
            for (_index, node) in &behavior.nodes {
                if node.behavior_type == BehaviorNodeType::SkillTree {
                    let mut rc: Option<(BehaviorType, Uuid, Uuid, String)> = None;
                    let mut parent_id = node.id;

                    for _lvl in 0..=skill_level {
                        for (id1, c1, id2, c2) in &behavior.connections {
                            if *id1 == parent_id && *c1 == BehaviorNodeConnector::Bottom {
                                for (uuid, node) in &behavior.nodes {
                                    if *uuid == *id2 {
                                        rc = Some((
                                            BehaviorType::Items,
                                            behavior.id,
                                            node.id,
                                            "script".to_string(),
                                        ));
                                        parent_id = node.id;
                                        break;
                                    }
                                }
                                break;
                            } else if *id2 == parent_id && *c2 == BehaviorNodeConnector::Bottom {
                                for (uuid, node) in &behavior.nodes {
                                    if *uuid == *id1 {
                                        rc = Some((
                                            BehaviorType::Items,
                                            behavior.id,
                                            node.id,
                                            "script".to_string(),
                                        ));
                                        parent_id = node.id;
                                        break;
                                    }
                                }
                                break;
                            }
                        }
                    }

                    return rc;
                }
            }
        }
    }
    None
}
