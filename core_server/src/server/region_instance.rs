extern crate ref_thread_local;
use ref_thread_local::{RefThreadLocal};

use crate::{prelude::*};
use rhai::{Engine, AST, Scope};

pub struct RegionInstance<'a> {
    // Game data
    pub region_data                 : GameRegionData,
    pub region_behavior             : Vec<GameBehaviorData>,

    pub behaviors                   : FxHashMap<Uuid, GameBehaviorData>,
    pub systems                     : FxHashMap<Uuid, GameBehaviorData>,
    pub items                       : FxHashMap<Uuid, GameBehaviorData>,
    pub spells                      : FxHashMap<Uuid, GameBehaviorData>,
    pub game_data                   : GameBehaviorData,
    pub scripts                     : FxHashMap<String, String>,

    // For faster lookup
    pub system_names                : Vec<String>,
    pub system_ids                  : Vec<Uuid>,
    pub area_ids                    : Vec<Uuid>,

    // All nodes
    nodes                           : FxHashMap<BehaviorNodeType, NodeCall>,

    /// The script engine
    pub engine                      : Engine,
    /// Script ast's, id is (BehaviorType, BehaviorId, BehaviorNodeID, AtomParameterID)
    pub ast                         : FxHashMap<(BehaviorType, Uuid, Uuid, String), AST>,

    // Character instances
    pub instances                   : Vec<BehaviorInstance>,
    pub scopes                      : Vec<rhai::Scope<'a>>,

    /// The loot in the region
    pub loot                        : FxHashMap<(isize, isize), Vec<Item>>,

    /// During action execution for regions this indicates the calling behavior index
    pub curr_action_inst_index      : Option<usize>,

    /// If the execute_node call has indirection, this is set to the original index
    pub curr_redirected_inst_index  : Option<usize>,

    /// Player game scopes
    pub game_player_scopes          : FxHashMap<usize, Scope<'a>>,

    /// The index of the game instance
    game_instance_index             : Option<usize>,

    /// The displacements for this region
    pub displacements               : HashMap<(isize, isize), TileData>,

    // Used by ticks for state memory

    /// Current characters per region
    //pub characters                  : FxHashMap<Uuid, Vec<CharacterData>>,
    // Characters instance indices in a given area
    pub area_characters             : FxHashMap<usize, Vec<usize>>,
    // The character instances from the previous tick, used to figure out onEnter, onLeave etc events
    pub prev_area_characters        : FxHashMap<usize, Vec<usize>>,

    // The current move direction of the player
    pub action_direction_text       : String,

    // The current subject (inventory item etc.) of the player
    pub action_subject_text         : String,

    // Identifie the currently executing loot item
    pub curr_loot_item              : Option<(isize, isize, usize)>,

    // Identify the currently executing inventory item index
    pub curr_inventory_index        : Option<usize>,

    // The current player scope (if swapped out during item execution)
    pub curr_player_scope           : Scope<'a>,

    // The currently executing behavior tree id
    pub curr_executing_tree         : Uuid,

    // These are fields which provide debug feedback while running and are only used in the editors debug mode

    // The behavior id to debug, this is send from the server
    debug_behavior_id               : Option<Uuid>,

    // We are debugging the current tick characters
    is_debugging                    : bool,

    pub messages                    : Vec<(String, MessageType)>,
    pub executed_connections        : Vec<(BehaviorType, Uuid, BehaviorNodeConnector)>,
    pub script_errors               : Vec<((Uuid, Uuid, String), (String, Option<u32>))>,

    // Region settings

    pub pixel_based_movement        : bool,

    /// Tick count used for timing
    pub dealt_damage_success        : bool,

    /// Respawns the given chararacter uuid at the given tick count
    pub respawn_instance            : FxHashMap<Uuid, (usize, CharacterInstanceData)>,

    // Game settings

    screen_size                     : (i32, i32),
    def_square_tile_size            : i32,

    pub gear_slots                  : Vec<String>,
    pub weapon_slots                : Vec<String>,

    pub skill_trees                 : FxHashMap<String, Vec<(i32, String, String)>>,

    pub ticks_per_minute            : usize,

    // Variable names

    pub primary_currency            : String,
    pub hitpoints                   : String,
    pub max_hitpoints               : String,
}

impl RegionInstance<'_> {

    pub fn new() -> Self {
        let mut engine = Engine::new();

        // Variable resolver for d??? -> random(???)
        #[allow(deprecated)]
        engine.on_var(|name, _index, _context| {
            if name.starts_with("d") {
                let mut s = name.to_string();
                s.remove(0);
                if let Some(n) = s.parse::<i32>().ok() {
                    let mut util = UTILITY.borrow_mut();
                    let random = util.rng.gen_range(1..=n) as f32;
                    return Ok(Some(random.into()));
                }
            }
            Ok(None)
        });

        engine.register_fn("roll", |exp: &str| -> i32 {
            let mut util = UTILITY.borrow_mut();
            if let Some(rc) = util.roll(exp).ok() {
                rc
            } else {
                1
            }
        });

        engine.register_fn("get_sheet", || -> Sheet {
            let data = &REGION_DATA.borrow_mut()[*CURR_INST.borrow()];
            data.sheets[data.curr_index].clone()
        });

        engine.register_fn("set_sheet", |sheet: Sheet| {
            let data = &mut REGION_DATA.borrow_mut()[*CURR_INST.borrow()];
            data.sheets[data.curr_index] = sheet;
        });

        engine.register_fn("inventory_add", |mut sheet: Sheet, item_name: &str| -> Sheet {
            inventory_add(&mut sheet, item_name, 1, &mut ITEMS.borrow_mut());
            sheet
        });

        Sheet::register(&mut engine);
        Currency::register(&mut engine);

        script_register_message_api(&mut engine);
        script_register_inventory_api(&mut engine);
        script_register_spells_api(&mut engine);
        script_register_gear_api(&mut engine);
        script_register_weapons_api(&mut engine);
        script_register_experience_api(&mut engine);
        script_register_date_api(&mut engine);
        script_register_failure_enum_api(&mut engine);

        // Display f64 as ints
        use pathfinding::num_traits::ToPrimitive;
        engine.register_fn("to_string", |x: f32| format!("{}", x.to_isize().unwrap()));

        let mut nodes : FxHashMap<BehaviorNodeType, NodeCall> = FxHashMap::default();

        //nodes.insert(BehaviorNodeType::Expression, expression);
        //nodes.insert(BehaviorNodeType::Script, script);
        //nodes.insert(BehaviorNodeType::Pathfinder, pathfinder);
        //nodes.insert(BehaviorNodeType::Lookout, lookout);
        //nodes.insert(BehaviorNodeType::CloseIn, close_in);
        //nodes.insert(BehaviorNodeType::CallSystem, call_system);
        //nodes.insert(BehaviorNodeType::CallBehavior, call_behavior);
        //nodes.insert(BehaviorNodeType::HasTarget, has_target);
        //nodes.insert(BehaviorNodeType::Untarget, untarget);
        nodes.insert(BehaviorNodeType::MagicDamage, magic_damage);
        nodes.insert(BehaviorNodeType::DealDamage, deal_damage);
        nodes.insert(BehaviorNodeType::TakeDamage, take_damage);
        nodes.insert(BehaviorNodeType::DropInventory, drop_inventory);
        nodes.insert(BehaviorNodeType::Effect, effect);
        //nodes.insert(BehaviorNodeType::Audio, audio);
        nodes.insert(BehaviorNodeType::Heal, heal);
        nodes.insert(BehaviorNodeType::TakeHeal, take_heal);
        nodes.insert(BehaviorNodeType::Respawn, respawn);
        nodes.insert(BehaviorNodeType::SetLevelTree, set_level_tree);
        nodes.insert(BehaviorNodeType::Schedule, schedule);
        //nodes.insert(BehaviorNodeType::HasState, has_state);

        nodes.insert(BehaviorNodeType::OverlayTiles, overlay_tiles);

        // nodes.insert(BehaviorNodeType::Move, player_move);
        // nodes.insert(BehaviorNodeType::Screen, screen);
        // nodes.insert(BehaviorNodeType::Widget, widget);
        //nodes.insert(BehaviorNodeType::Message, message);
        // nodes.insert(BehaviorNodeType::Action, player_action);
        //nodes.insert(BehaviorNodeType::Take, player_take);
        // nodes.insert(BehaviorNodeType::Drop, player_drop);
        // nodes.insert(BehaviorNodeType::Target, player_target);
        //nodes.insert(BehaviorNodeType::Equip, player_equip);
        //nodes.insert(BehaviorNodeType::MagicTarget, magic_target);
        //nodes.insert(BehaviorNodeType::LightItem, light_item);
        nodes.insert(BehaviorNodeType::SetItemTile, set_item_tile);
        //nodes.insert(BehaviorNodeType::RandomWalk, random_walk);
        //nodes.insert(BehaviorNodeType::MultiChoice, multi_choice);
        //nodes.insert(BehaviorNodeType::Sell, sell);
        // nodes.insert(BehaviorNodeType::LockTree, lock_tree);
        // nodes.insert(BehaviorNodeType::UnlockTree, unlock_tree);
        // nodes.insert(BehaviorNodeType::SetState, set_state);
        nodes.insert(BehaviorNodeType::Teleport, teleport);

        //nodes.insert(BehaviorNodeType::Always, always);
        //nodes.insert(BehaviorNodeType::InsideArea, inside_area);
        //nodes.insert(BehaviorNodeType::EnterArea, enter_area);
        //nodes.insert(BehaviorNodeType::LeaveArea, leave_area);
        //nodes.insert(BehaviorNodeType::TeleportArea, teleport_area);
        //nodes.insert(BehaviorNodeType::MessageArea, message_area);
        //nodes.insert(BehaviorNodeType::AudioArea, audio_area);
        //nodes.insert(BehaviorNodeType::LightArea, light_area);
        //nodes.insert(BehaviorNodeType::ActionArea, action);

        nodes.insert(BehaviorNodeType::SkillTree, skill_tree);
        nodes.insert(BehaviorNodeType::SkillLevel, skill_level);

        Self {
            region_data                     : GameRegionData::new(),
            region_behavior                 : vec![],

            behaviors                       : FxHashMap::default(),
            systems                         : FxHashMap::default(),
            items                           : FxHashMap::default(),
            spells                          : FxHashMap::default(),
            game_data                       : GameBehaviorData::new(),
            scripts                         : FxHashMap::default(),

            system_names                    : vec![],
            system_ids                      : vec![],
            area_ids                        : vec![],

            engine,
            ast                             : FxHashMap::default(),
            nodes,

            instances                       : vec![],
            scopes                          : vec![],

            loot                            : FxHashMap::default(),

            curr_action_inst_index          : None,

            curr_redirected_inst_index      : None,

            game_player_scopes              : FxHashMap::default(),

            game_instance_index             : None,

            displacements                   : HashMap::new(),

            //characters                      : FxHashMap::default(),
            area_characters                 : FxHashMap::default(),
            prev_area_characters            : FxHashMap::default(),

            action_direction_text           : "".to_string(),
            action_subject_text             : "".to_string(),

            curr_loot_item                  : None,
            curr_inventory_index            : None,
            curr_player_scope               : Scope::new(),

            curr_executing_tree             : Uuid::new_v4(),

            debug_behavior_id               : None,
            is_debugging                    : false,

            messages                        : vec![],
            executed_connections            : vec![],
            script_errors                   : vec![],

            pixel_based_movement            : true,

            dealt_damage_success            : false,

            respawn_instance                : FxHashMap::default(),

            screen_size                     : (1024, 608),
            def_square_tile_size            : 32,

            weapon_slots                    : vec![],
            gear_slots                      : vec![],

            ticks_per_minute                : 4,

            skill_trees                     : FxHashMap::default(),

            // Variable names
            primary_currency                : "".to_string(),
            hitpoints                       : "".to_string(),
            max_hitpoints                   : "".to_string(),
        }
    }

    /// Game tick
    pub fn tick(&mut self) -> Vec<Message> {

        self.messages = vec![];
        self.prev_area_characters = self.area_characters.clone();
        self.area_characters = FxHashMap::default();

        let mut messages = vec![];

        let character_instances_len;
        {
            let data = &mut REGION_DATA.borrow_mut()[*CURR_INST.borrow()];
            data.characters.clear();
            data.area_characters.clear();
            data.prev_area_characters.clear();
            data.lights.clear();

            character_instances_len = data.character_instances.len();
        }

        let tick_time = self.get_time();

        // Check if we need to respawn something

        if self.respawn_instance.is_empty() == false {
            for (id, (tick, data)) in &self.respawn_instance.clone() {
                if *tick <= *TICK_COUNT.borrow() as usize {
                    self.create_behavior_instance(*id, false, Some(data.clone()));
                    self.respawn_instance.remove(id);
                }
            }
        }

        // Execute behaviors
        for inst_index in 0..character_instances_len {

            self.messages = vec![];
            self.executed_connections = vec![];
            self.script_errors = vec![];

            let state;
            let instance_type;

            let sleeping;

            {
                let data = &mut REGION_DATA.borrow_mut()[*CURR_INST.borrow()];
                data.curr_index = inst_index;

                data.character_instances[inst_index].audio = vec![];
                data.character_instances[inst_index].multi_choice_data = vec![];

                state = data.character_instances[inst_index].state;
                instance_type = data.character_instances[inst_index].instance_type;

                if self.pixel_based_movement == true {
                    if  data.character_instances[data.curr_index].old_position.is_some() {
                        data.character_instances[data.curr_index].curr_transition_time += 1;

                        if data.character_instances[data.curr_index].curr_transition_time > data.character_instances[data.curr_index].max_transition_time {
                            data.character_instances[data.curr_index].old_position = None;
                            data.character_instances[data.curr_index].curr_transition_time = 0;
                        }
                    }
                }

                if data.character_instances[inst_index].sleep_cycles > 0 {
                    data.character_instances[inst_index].sleep_cycles -= 1;
                    sleeping = true;
                } else {
                    sleeping = false;
                }
            }

            // Skip Sleep cycles
            if sleeping == false {
                // Purged: Skip
                if state == BehaviorInstanceState::Purged {
                    continue;
                }

                // Killed: NPC Skip
                if state == BehaviorInstanceState::Killed && instance_type == BehaviorInstanceType::NonPlayerCharacter {
                    continue;
                }

                // Are we debugging this character ?
                self.is_debugging = Some(self.instances[inst_index].behavior_id) == self.debug_behavior_id;

                if instance_type == BehaviorInstanceType::NonPlayerCharacter {

                    let mut execute_trees = true;

                    {
                        let data = &mut REGION_DATA.borrow_mut()[*CURR_INST.borrow()];

                        // Check if this NPC has active communication
                        if data.character_instances[inst_index].communication.is_empty() == false {
                            let mut com_to_drop : Option<usize> = None;

                            for c_index in 0..data.character_instances[inst_index].communication.len() {
                                if data.character_instances[inst_index].communication[c_index].end_time < *DATE.borrow() {

                                    // Drop this communication for the NPC
                                    com_to_drop = Some(c_index);

                                    // Remove the communication for the Player
                                    let player_index = data.character_instances[inst_index].communication[c_index].player_index;
                                    data.character_instances[player_index].communication = vec![];
                                    data.character_instances[player_index].multi_choice_data = vec![];

                                    break;
                                }
                            }

                            if let Some(index) = com_to_drop {
                                data.character_instances[inst_index].communication.remove(index);
                            }

                            // Communication is ongoing, dont do anything
                            if data.character_instances[inst_index].communication.is_empty() == false {
                                execute_trees = false;
                            }
                        }
                    }

                    if execute_trees {
                        // Execute trees of an NPC

                        // Has a locked tree ?
                        if let Some(locked_tree) = self.instances[inst_index].locked_tree {
                                self.execute_node(inst_index, locked_tree, None);
                        } else {
                            // Unlocked, execute all valid trees
                            let trees;
                            let behavior_id;
                            {
                                let data = &REGION_DATA.borrow()[*CURR_INST.borrow()];
                                trees = data.character_instances[inst_index].tree_ids.clone();
                                behavior_id = data.character_instances[inst_index].behavior_id;
                            }
                            for node_id in &trees {

                                // Only execute trees here with an "Always" execute setting (0)
                                if let Some(value)= get_node_value((behavior_id, *node_id, "execute"), self, BehaviorType::Behaviors) {
                                    if let Some(value) = value.to_integer() {
                                        if value != 0 {
                                            continue;
                                        }
                                    }
                                }
                                //self.scopes[inst_index].set_value("date", self.date.clone());
                                //self.execute_node(inst_index, node_id.clone(), None);
                                execute_node(behavior_id, *node_id, &mut BEHAVIORS.borrow_mut());
                            }
                        }
                    }
                } else
                if instance_type == BehaviorInstanceType::Player {

                    // Execute the tree which matches the current action

                    let mut tree_id: Option<Uuid> = None;
                    let action;
                    {
                        let data = &mut REGION_DATA.borrow_mut()[*CURR_INST.borrow()];
                        action = data.character_instances[inst_index].action.clone();
                        // DEBUG INCOMING ACTION
                        // println!("{:?}", action);
                    }
                    if let Some(action) = &action {
                        if action.direction != PlayerDirection::None {

                            // A directed action ( Move / Look - North etc)

                            if action.action.to_lowercase() == "cast" && action.spell.is_some() {
                                // Cast spell

                                let mut to_cast : Option<(Uuid, Uuid)> = None;
                                if let Some(spell_name) = &action.spell {
                                    let name = spell_name.to_lowercase();
                                    for (behavior_id, spell) in&self.spells {
                                        if spell.name.to_lowercase() == name {
                                            for (node_id, node) in &spell.nodes {
                                                if node.behavior_type == BehaviorNodeType::BehaviorTree && node.name.to_lowercase() == "cast" {
                                                    to_cast = Some((*behavior_id, *node_id));
                                                }
                                            }
                                        }
                                    }
                                }

                                if let Some(to_cast) = to_cast {
                                    execute_node(to_cast.0, to_cast.1, &mut SPELLS.borrow_mut());
                                }

                            } else {
                                for id in &self.instances[inst_index].tree_ids {
                                    if let Some(behavior) = self.get_behavior(self.instances[inst_index].behavior_id, BehaviorType::Behaviors) {
                                        if let Some(node) = behavior.nodes.get(&id) {
                                            if node.name == action.action {
                                                tree_id = Some(*id);
                                                break;
                                            }
                                        }
                                    }
                                }

                                if let Some(tree_id) = tree_id {
                                    execute_node(self.instances[inst_index].behavior_id, tree_id, &mut BEHAVIORS.borrow_mut());
                                } else {
                                    println!("Cannot find valid tree for directed action {}", action.action);
                                }
                            }
                        } else
                        if let Some(inventory_index) = &action.inventory_index {

                            // An action on an inventory item index

                            let index = *inventory_index as usize;

                            // Get the item and set the state if any
                            if let Some(item) = get_inventory_item_at(index, true) {
                                let mut to_execute = vec![];

                                // Get the behavior trees to execute
                                if let Some(item_behavior) = self.get_behavior(item.id, BehaviorType::Items) {
                                    for (id, node) in &item_behavior.nodes {
                                        if node.behavior_type == BehaviorNodeType::BehaviorTree {
                                            if node.name == action.action {
                                                to_execute.push((item_behavior.id, *id));
                                            }
                                        }
                                    }
                                }

                                // Execute them
                                if to_execute.is_empty() == false {
                                    for (behavior_id, node_id) in to_execute {
                                        execute_node(behavior_id, node_id, &mut ITEMS.borrow_mut());
                                    }
                                    set_inventory_item_state_at(index);
                                } else {
                                    // If we cannot find the tree on the item, look for it on the player
                                    for id in &self.instances[inst_index].tree_ids {
                                        let data: &mut RegionData = &mut REGION_DATA.borrow_mut()[*CURR_INST.borrow()];
                                        if let Some(behavior) = self.get_behavior(data.character_instances[inst_index].behavior_id, BehaviorType::Behaviors) {
                                            if let Some(node) = behavior.nodes.get(&id) {
                                                if node.name == action.action {
                                                    to_execute.push((data.character_instances[inst_index].behavior_id, *id));
                                                    break;
                                                }
                                            }
                                        }
                                    }

                                    if to_execute.is_empty() == false {
                                        for (behavior_id, node_id) in to_execute {
                                            execute_node(behavior_id, node_id, &mut BEHAVIORS.borrow_mut());
                                        }
                                    } else {
                                        println!("Cannot find valid tree for directed action {}", action.action);
                                    }
                                }
                            }
                        } else
                        if let Some(uuid) = &action.multi_choice_uuid {
                            // Multi Choice Answer

                            let mut communication_id : Option<(Uuid, Uuid)> = None;

                            {
                                let data = &mut REGION_DATA.borrow_mut()[*CURR_INST.borrow()];
                                if data.character_instances[inst_index].communication.is_empty() == false {

                                    //let npc_index = data.character_instances[inst_index].communication[0].npc_index;
                                    communication_id = Some(data.character_instances[inst_index].communication[0].npc_behavior_id);
                                    data.character_instances[inst_index].multi_choice_answer = Some(*uuid);
                                }
                            }

                            if let Some(behavior_id) = communication_id {
                                execute_node(behavior_id.0, behavior_id.1, &mut BEHAVIORS.borrow_mut());
                            }

                            let data = &mut REGION_DATA.borrow_mut()[*CURR_INST.borrow()];

                            data.character_instances[inst_index].communication = vec![];
                            data.character_instances[inst_index].target_instance_index = None;
                        }

                        {
                            let data = &mut REGION_DATA.borrow_mut()[*CURR_INST.borrow()];
                            data.character_instances[inst_index].action = None;
                        }
                    }
                    // Characters do not lock on targets
                    self.instances[inst_index].target_instance_index = None;
                }
            }

            // Extract the script messages for this instance
            if let Some(mess) = self.scopes[inst_index].get_mut("message") {
                if let Some(mut message) = mess.write_lock::<ScriptMessageCmd>() {
                    if message.messages.is_empty() == false {
                        let my_name = self.instances[inst_index].name.clone();
                        for m in &message.messages {
                            match m {
                                ScriptMessage::Status(value) => {
                                    self.instances[inst_index].messages.push( MessageData {
                                        message_type        : MessageType::Status,
                                        message             : value.clone(),
                                        from                : my_name.clone(),
                                        right               : None,
                                        center              : None,
                                        buffer              : None,
                                    })
                                },
                                ScriptMessage::Debug(value) => {
                                    self.instances[inst_index].messages.push( MessageData {
                                        message_type        : MessageType::Debug,
                                        message             : value.clone(),
                                        from                : my_name.clone(),
                                        right               : None,
                                        center              : None,
                                        buffer              : None,
                                    })
                                },
                                ScriptMessage::Error(value) => {
                                    self.instances[inst_index].messages.push( MessageData {
                                        message_type        : MessageType::Error,
                                        message             : value.clone(),
                                        from                : my_name.clone(),
                                        right               : None,
                                        center              : None,
                                        buffer              : None,
                                    })
                                }
                            }
                        }
                    }
                    message.clear();
                }
            }

            // Inventory Actions

            let mut to_add = vec![];
            let mut to_equip = vec![];
            let mut to_equip_queued = vec![];

            // Check if we have to add items to the inventory and clone it for sending to the client
            if let Some(i) = self.scopes[inst_index].get_mut("inventory") {
                if let Some(mut inv) = i.write_lock::<Inventory>() {

                    // Add items
                    if inv.items_to_add.is_empty() == false {
                        let items_to_add = inv.items_to_add.clone();
                        for data in &items_to_add {
                            for (_id, behavior) in &mut self.items {

                                let mut added = false;

                                for item in &mut inv.items {
                                    if item.name == *data.0 {
                                        item.amount += data.1 as i32;
                                        added = true;
                                        break;
                                    }
                                }

                                if added == false {
                                    let mut tile_data : Option<TileData> = None;
                                    let mut sink : Option<PropertySink> = None;

                                    // Get the default tile for the item
                                    for (_index, node) in &behavior.nodes {
                                        if node.behavior_type == BehaviorNodeType::BehaviorType {
                                            if let Some(value) = node.values.get(&"tile".to_string()) {
                                                tile_data = value.to_tile_data();
                                            }
                                            if let Some(value) = node.values.get(&"settings".to_string()) {
                                                if let Some(str) = value.to_string() {
                                                    let mut s = PropertySink::new();
                                                    s.load_from_string(str.clone());
                                                    sink = Some(s);
                                                }
                                            }
                                        }
                                    }

                                    if behavior.name == *data.0 {
                                        let mut item = Item::new(behavior.id, behavior.name.clone());
                                        item.item_type = "gear".to_string();
                                        item.tile = tile_data;
                                        item.amount = data.1 as i32;
                                        item.stackable = 1;

                                        // Add state ?

                                        let mut states_to_execute = vec![];

                                        if let Some(sink) = sink {
                                            if let Some(state) = sink.get("state") {
                                                if let Some(value) = state.as_bool() {
                                                    if value == true {
                                                        // TODO item.state = Some(ScopeBuffer::new());
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
                                                    }
                                                }
                                            }
                                            item.read_from_sink(&sink);
                                        }

                                        to_add.push((item, states_to_execute));
                                        break;
                                    }
                                } else {
                                    break;
                                }
                            }
                        }
                        inv.items_to_add = vec![];
                    }

                    // Equip an item ?
                    if inv.items_to_equip.is_empty() == false {
                        for index in 0..inv.items_to_equip.len() {
                            let name = inv.items_to_equip[index].clone();
                            // Item is in the inventory ?
                            let removed_item = inv.remove_item_by_name(name.clone());
                            if let Some(item) = removed_item {
                                to_equip.push(item);
                            } else {
                                // Not in the inventory, has to be queued
                                to_equip_queued.push(name);
                            }
                        }
                        inv.items_to_equip = vec![];
                    }
                }
            }

            // Add new items
            for (mut item, states_to_execute) in to_add {
                for (item_id, node_id) in states_to_execute {
                    let curr_scope = self.scopes[inst_index].clone();
                    self.scopes[inst_index] = Scope::new();
                    self.execute_item_node(inst_index, item_id, node_id);
                    let scope = self.scopes[inst_index].clone();
                    self.scopes[inst_index] = curr_scope;
                    let mut buffer = ScopeBuffer::new();
                    buffer.read_from_scope(&scope);
                    // TODO item.state = Some(buffer);
                }
                if let Some(mess) = self.scopes[inst_index].get_mut("inventory") {
                    if let Some(mut inv) = mess.write_lock::<Inventory>() {
                        // Test if the item is queued to be equipped
                        if let Some(queued_index) = to_equip_queued.iter().position(|name| *name == item.name) {
                            to_equip_queued.remove(queued_index);
                            to_equip.push(item);
                        } else {
                            inv.add_item(item);
                        }
                    }
                }
            }

            // Equip items
            let mut to_add_back_to_inventory: Vec<Item> = vec![];
            for item in to_equip {
                let item_type = item.item_type.clone().to_lowercase();
                if let Some(slot) = item.slot.clone() {
                    if item_type == "weapon" {
                        if let Some(mess) = self.scopes[inst_index].get_mut("weapons") {
                            if let Some(mut weapons) = mess.write_lock::<Weapons>() {
                                // Remove existing item in the slot
                                if let Some(w) = weapons.slots.remove(&slot) {
                                    to_add_back_to_inventory.push(w);
                                }
                                // Insert the new weapon into the slot
                                weapons.slots.insert(slot, item);
                            }
                        }
                    } else
                    if item_type == "gear" {
                        if let Some(mess) = self.scopes[inst_index].get_mut("gear") {
                            if let Some(mut gear) = mess.write_lock::<Gear>() {
                                // Remove existing item in the slot
                                if let Some(g) = gear.slots.remove(&slot) {
                                    to_add_back_to_inventory.push(g);
                                }
                                // Insert the new gear into the slot
                                gear.slots.insert(slot, item);
                            }
                        }
                    }
                }
            }

            // Add removed items in the equipped slot(s) back into the inventory
            if to_add_back_to_inventory.is_empty() == false {
                if let Some(mess) = self.scopes[inst_index].get_mut("inventory") {
                    if let Some(mut inv) = mess.write_lock::<Inventory>() {
                        for item in to_add_back_to_inventory {
                            inv.items.push(item);
                        }
                    }
                }
            }

            // If we are debugging this instance, send the debug data
            if Some(self.instances[inst_index].behavior_id) == self.debug_behavior_id {
                let debug = BehaviorDebugData {
                    executed_connections    : self.executed_connections.clone(),
                    script_errors           : self.script_errors.clone(),
                };
                messages.push(Message::DebugData(debug));
            }

            // Add to the characters

            let data: &mut RegionData = &mut REGION_DATA.borrow_mut()[*CURR_INST.borrow()];

            if let Some(position) = &data.character_instances[data.curr_index].position {
                if let Some(tile) = self.instances[inst_index].tile.clone() {
                    let character = CharacterData {
                        position                : position.clone(),
                        old_position            : data.character_instances[data.curr_index].old_position.clone(),
                        max_transition_time     : data.character_instances[data.curr_index].max_transition_time,
                        curr_transition_time    : data.character_instances[data.curr_index].curr_transition_time,
                        tile,
                        name                    : data.character_instances[data.curr_index].name.clone(),
                        id                      : data.character_instances[data.curr_index].id,
                        index                   : inst_index,
                        effects                 : data.character_instances[data.curr_index].effects.clone(),
                     };
                     if let Some(list) = data.characters.get_mut(&position.region) {
                         list.push(character);
                     } else {
                         data.characters.insert(position.region, vec![character]);
                     }
                }
                self.instances[inst_index].effects = vec![];
            }

            // Check the inventory for lights
            let lights = get_inventory_lights(data);

            for mut light in lights {
                if let Some(position) = &data.character_instances[inst_index].position {
                    light.position = (position.x, position.y);
                }
                data.lights.push(light);
            }
        }

        // Parse the loot and add the lights
        {
            let data: &mut RegionData = &mut REGION_DATA.borrow_mut()[*CURR_INST.borrow()];
            for (position, loot) in &data.loot {
                for item in loot {
                    if let Some(state) = &item.state {
                        if let Some(light) = &state.light {
                            let mut light_clone = light.clone();
                            light_clone.position = *position;
                            data.lights.push(light_clone);
                        }
                    }
                }
            }
        }

        // Execute region area behaviors

        let mut to_execute: Vec<(Uuid, usize, Uuid)> = vec![];
        {
            let data = &mut REGION_DATA.borrow_mut()[*CURR_INST.borrow()];
            data.displacements = FxHashMap::default();
            for area_index in 0..data.region_data.areas.len() {
                for (node_id, node) in &data.region_area_behavior[area_index].nodes {
                    if node.behavior_type == BehaviorNodeType::InsideArea || node.behavior_type == BehaviorNodeType::EnterArea || node.behavior_type == BehaviorNodeType::LeaveArea || node.behavior_type == BehaviorNodeType::Always {
                        to_execute.push((data.region_data.id, area_index, *node_id));
                    }
                }
            }
        }

        for tuple in to_execute {
            execute_area_node(tuple.0, tuple.1, tuple.2);
        }

        // Parse the player characters and generate updates

        for inst_index in 0..character_instances_len {

            let mut skills = Skills::new();
            let mut experience = Experience::new();

            // Clone the skills for sending it to the client
            if let Some(s) = self.scopes[inst_index].get("skills") {
                if let Some(sk) = s.read_lock::<Skills>() {
                    skills = sk.clone();
                }
            }

            // Clone the experience for sending it to the client
            if let Some(s) = self.scopes[inst_index].get("experience") {
                if let Some(exp) = s.read_lock::<Experience>() {
                    experience = exp.clone();
                }
            }

            // Purge invalid target indices
            if let Some(target_index) = self.instances[inst_index].target_instance_index {
                if self.instances[target_index].state.is_dead() {
                    self.instances[inst_index].target_instance_index = None;
                }
            }

            let mut send_update = false;

            // Send update if this is a player and no editor debugging
            if self.instances[inst_index].instance_type == BehaviorInstanceType::Player && self.debug_behavior_id.is_none() {
                send_update = true;
            } else
            // Otherwise send this update if this is the current character being debugged in the editor
            if Some(self.instances[inst_index].behavior_id) == self.debug_behavior_id {
                send_update = true;
            }

            if send_update {

                {
                    let data: &mut RegionData = &mut REGION_DATA.borrow_mut()[*CURR_INST.borrow()];
                    if data.character_instances[inst_index].state == BehaviorInstanceState::Purged {
                        continue;
                    }
                }

                let old_screen_id;
                let mut game_locked_tree : Option<Uuid> = None;

                {
                    let data: &mut RegionData = &mut REGION_DATA.borrow_mut()[*CURR_INST.borrow()];

                    old_screen_id = data.character_instances[inst_index].curr_player_screen_id;
                    data.curr_player_inst_index = inst_index;
                    if let Some(game_inst_index) = data.game_instance_index {
                        if let Some(locked_tree) = data.character_instances[game_inst_index].locked_tree {
                            game_locked_tree = Some(locked_tree);
                        }
                    }
                }

                let mut screen_script_name : Option<String> = None;
                let mut screen_scripts : Option<FxHashMap<String, String>> = None;
                let mut widgets : Vec<String> = vec![];

                // Execute the game behavior
                if let Some(locked_tree) = game_locked_tree {
                    execute_node(self.game_data.id, locked_tree, &mut GAME_BEHAVIOR.borrow_mut());
                }

                // Send screen scripts ?

                let data: &mut RegionData = &mut REGION_DATA.borrow_mut()[*CURR_INST.borrow()];

                if data.character_instances[inst_index].send_screen_scripts == false {
                    screen_scripts = Some(self.scripts.clone());
                    data.character_instances[inst_index].send_screen_scripts =  true;
                }

                // Check if we need to send a new screen script name

                if let Some(new_screen_id) = &data.character_instances[inst_index].curr_player_screen_id {
                    if let Some(old_screen_id) = &old_screen_id {
                        if new_screen_id != old_screen_id {
                            screen_script_name = Some(data.character_instances[inst_index].curr_player_screen.clone());
                            widgets = data.character_instances[inst_index].curr_player_widgets.clone();
                        }
                    } else {
                        screen_script_name = Some(data.character_instances[inst_index].curr_player_screen.clone());
                        widgets = data.character_instances[inst_index].curr_player_widgets.clone();
                    }
                }

                let mut region        : Option<GameRegionData> = None;
                let mut characters    : Vec<CharacterData> = vec![];
                let mut displacements : FxHashMap<(isize, isize), TileData> = FxHashMap::default();
                let mut scope_buffer = ScopeBuffer::new();

                let mut needs_transfer_to: Option<Uuid> = None;
                if let Some(position) = &data.character_instances[inst_index].position.clone() {

                    if position.region != data.region_data.id {
                        // We need to transfer the character to a new region
                        needs_transfer_to = Some(position.region);
                    } else
                    // Check if the character is in a region we did not send to the client yet OR if the editor is debugging
                    if data.character_instances[inst_index].regions_send.contains(&position.region) == false || self.debug_behavior_id.is_some() {
                        region = Some(data.region_data.clone());
                        data.character_instances[inst_index].regions_send.insert(position.region);
                    }
                    // Copy the displacements
                    displacements = data.displacements.clone();

                    // Send the characters of the client region
                    if let Some(chars) = data.characters.get(&position.region) {
                        characters = chars.clone();
                    }

                    scope_buffer.read_from_scope(&self.scopes[inst_index]);
                }

                let update = GameUpdate{
                    id                      : data.character_instances[inst_index].id,
                    screen_size             : self.screen_size,
                    def_square_tile_size    : self.def_square_tile_size,
                    position                : data.character_instances[inst_index].position.clone(),
                    old_position            : data.character_instances[inst_index].old_position.clone(),
                    max_transition_time     : data.character_instances[inst_index].max_transition_time,
                    curr_transition_time    : data.character_instances[inst_index].curr_transition_time,
                    tile                    : data.character_instances[inst_index].tile.clone(),
                    sheet                   : data.sheets[inst_index].clone(),
                    screen_script_name,
                    screen_scripts,
                    widgets,
                    region,
                    lights                  : data.lights.clone(),
                    displacements,
                    characters,
                    loot                    : data.loot.clone(),
                    messages                : data.character_instances[inst_index].messages.clone(),
                    audio                   : data.character_instances[inst_index].audio.clone(),
                    scope_buffer            : scope_buffer,
                    skills                  : skills.clone(),
                    experience              : experience.clone(),
                    multi_choice_data       : data.character_instances[inst_index].multi_choice_data.clone(),
                    communication           : data.character_instances[inst_index].communication.clone(),
                    date                    : DATE.borrow().clone()
                 };

                data.character_instances[inst_index].messages = vec![];

                if let Some(transfer_to) = needs_transfer_to {
                    // Serialize character
                    self.serialize_character_instance(inst_index);
                    messages.push(Message::TransferCharacter(transfer_to, data.character_instances[inst_index].clone(), data.sheets[inst_index].clone()));
                    // Purge the character
                    data.character_instances[inst_index].state = BehaviorInstanceState::Purged;
                    data.player_uuid_indices.remove(&data.character_instances[inst_index].id);
                }
                messages.push(Message::PlayerUpdate(update.id, update));
            } else {
                // This handles character region transfers for NPCs
                if let Some(position) = self.instances[inst_index].position.clone() {
                    let mut needs_transfer_to: Option<Uuid> = None;
                    if position.region != self.region_data.id {
                        // We need to transfer the character to a new region
                        needs_transfer_to = Some(position.region);
                    }

                    if let Some(transfer_to) = needs_transfer_to {
                        // Serialize character
                        self.serialize_character_instance(inst_index);
                        let data: &mut RegionData = &mut REGION_DATA.borrow_mut()[*CURR_INST.borrow()];
                        messages.push(Message::TransferCharacter(transfer_to, data.character_instances[inst_index].clone(), data.sheets[inst_index].clone()));
                        // Purge the character
                        data.character_instances[inst_index].state = BehaviorInstanceState::Purged;
                        data.player_uuid_indices.remove(&data.character_instances[inst_index].id);
                    }
                }
            }
        }

        messages
    }

    /// Executes the given node and follows the connection chain
    pub fn execute_node(&mut self, instance_index: usize, node_id: Uuid, redirection: Option<usize>) -> Option<BehaviorNodeConnector> {

        let mut connectors : Vec<BehaviorNodeConnector> = vec![];
        let mut connected_node_ids : Vec<Uuid> = vec![];
        let mut possibly_executed_connections : Vec<(BehaviorType, Uuid, BehaviorNodeConnector)> = vec![];

        let mut is_sequence = false;
        let mut rc : Option<BehaviorNodeConnector> = None;

        // Call the node and get the resulting BehaviorNodeConnector
        if let Some(behavior) = self.behaviors.get_mut(&self.instances[instance_index].behavior_id) {
            if let Some(node) = behavior.nodes.get_mut(&node_id) {

                // Handle special nodes
                if node.behavior_type == BehaviorNodeType::BehaviorTree || node.behavior_type == BehaviorNodeType::Linear {

                    if node.behavior_type == BehaviorNodeType::BehaviorTree {
                        self.curr_executing_tree = node.id;
                    }

                    connectors.push(BehaviorNodeConnector::Bottom1);
                    connectors.push(BehaviorNodeConnector::Bottom2);
                    connectors.push(BehaviorNodeConnector::Bottom);
                    connectors.push(BehaviorNodeConnector::Bottom3);
                    connectors.push(BehaviorNodeConnector::Bottom4);
                } else
                if node.behavior_type == BehaviorNodeType::Sequence {
                    connectors.push(BehaviorNodeConnector::Bottom1);
                    connectors.push(BehaviorNodeConnector::Bottom2);
                    connectors.push(BehaviorNodeConnector::Bottom);
                    connectors.push(BehaviorNodeConnector::Bottom3);
                    connectors.push(BehaviorNodeConnector::Bottom4);
                    is_sequence = true;
                } else {
                    if let Some(node_call) = self.nodes.get_mut(&node.behavior_type) {
                        let behavior_id = self.instances[instance_index].behavior_id.clone();
                        let idx = if redirection.is_some() { redirection.unwrap() } else { instance_index};
                        let connector = node_call(idx, (behavior_id, node_id), self, BehaviorType::Behaviors);
                        rc = Some(connector);
                        connectors.push(connector);
                    } else {
                        connectors.push(BehaviorNodeConnector::Bottom);
                    }
                }
            }
        }

        // Search the connections to check if we can find an ongoing node connection
        for connector in connectors {
            if let Some(behavior) = self.behaviors.get_mut(&self.instances[instance_index].behavior_id) {

                for c in &behavior.connections {
                    if c.0 == node_id && c.1 == connector {
                        connected_node_ids.push(c.2);
                        if is_sequence == false {
                            self.executed_connections.push((BehaviorType::Behaviors, c.0, c.1));
                        } else {
                            possibly_executed_connections.push((BehaviorType::Behaviors, c.0, c.1));
                        }
                    }
                }
            }
        }

        // And if yes execute it
        for (index, connected_node_id) in connected_node_ids.iter().enumerate() {

            // If this is a sequence, mark this connection as executed
            if is_sequence {
                self.executed_connections.push(possibly_executed_connections[index]);
            }

            if let Some(connector) = self.execute_node(instance_index, *connected_node_id, redirection) {
                if is_sequence {
                    // Inside a sequence break out if the connector is not Success
                    if connector == BehaviorNodeConnector::Fail || connector == BehaviorNodeConnector::Right {
                        break;
                    }
                }
            }
        }
        rc
    }

    /// Executes the given systems node and follows the connection chain
    pub fn execute_systems_node(&mut self, instance_index: usize, node_id: Uuid) -> Option<BehaviorNodeConnector> {

        let mut connectors : Vec<BehaviorNodeConnector> = vec![];
        let mut connected_node_ids : Vec<Uuid> = vec![];
        let mut possibly_executed_connections : Vec<(BehaviorType, Uuid, BehaviorNodeConnector)> = vec![];

        let mut is_sequence = false;
        let mut rc : Option<BehaviorNodeConnector> = None;

        // Call the node and get the resulting BehaviorNodeConnector
        if let Some(system) = self.systems.get_mut(&self.instances[instance_index].systems_id) {
            if let Some(node) = system.nodes.get_mut(&node_id) {

                // Handle special nodes
                if node.behavior_type == BehaviorNodeType::BehaviorTree || node.behavior_type == BehaviorNodeType::Linear {
                    connectors.push(BehaviorNodeConnector::Bottom1);
                    connectors.push(BehaviorNodeConnector::Bottom2);
                    connectors.push(BehaviorNodeConnector::Bottom);
                } else
                if node.behavior_type == BehaviorNodeType::Sequence {
                    connectors.push(BehaviorNodeConnector::Bottom1);
                    connectors.push(BehaviorNodeConnector::Bottom2);
                    connectors.push(BehaviorNodeConnector::Bottom);
                    connectors.push(BehaviorNodeConnector::Bottom3);
                    connectors.push(BehaviorNodeConnector::Bottom4);
                    is_sequence = true;
                } else {
                    if let Some(node_call) = self.nodes.get_mut(&node.behavior_type) {
                        let systems_id = self.instances[instance_index].systems_id.clone();
                        let connector = node_call(instance_index, (systems_id, node_id), self, BehaviorType::Systems);
                        rc = Some(connector);
                        connectors.push(connector);
                    } else {
                        connectors.push(BehaviorNodeConnector::Bottom);
                    }
                }
            }
        }

        // Search the connections to check if we can find an ongoing node connection
        for connector in connectors {
            if let Some(system) = self.systems.get_mut(&self.instances[instance_index].systems_id) {

                for c in &system.connections {
                    if c.0 == node_id && c.1 == connector {
                        connected_node_ids.push(c.2);
                        if is_sequence == false {
                            self.executed_connections.push((BehaviorType::Systems, c.0, c.1));
                        } else {
                            possibly_executed_connections.push((BehaviorType::Systems, c.0, c.1));
                        }
                    }
                }
            }
        }

        // And if yes execute it
        for (index, connected_node_id) in connected_node_ids.iter().enumerate() {

            // If this is a sequence, mark this connection as executed
            if is_sequence {
                self.executed_connections.push(possibly_executed_connections[index]);
            }

            if let Some(connector) = self.execute_systems_node(instance_index, *connected_node_id) {
                if is_sequence {
                    // Inside a sequence break out if the connector is not Success
                    if connector == BehaviorNodeConnector::Fail || connector == BehaviorNodeConnector::Right {
                        break;
                    }
                }
            }
        }
        rc
    }

    /// Executes the given item node and follows the connection chain
    pub fn execute_item_node(&mut self, instance_index: usize, item_id: Uuid, node_id: Uuid) -> Option<BehaviorNodeConnector> {

        let mut connectors : Vec<BehaviorNodeConnector> = vec![];
        let mut connected_node_ids : Vec<Uuid> = vec![];
        let mut possibly_executed_connections : Vec<(BehaviorType, Uuid, BehaviorNodeConnector)> = vec![];

        let mut is_sequence = false;
        let mut rc : Option<BehaviorNodeConnector> = None;

        // Call the node and get the resulting BehaviorNodeConnector
        if let Some(item) = self.items.get_mut(&item_id) {
            if let Some(node) = item.nodes.get_mut(&node_id) {

                // Handle special nodes
                if node.behavior_type == BehaviorNodeType::BehaviorTree || node.behavior_type == BehaviorNodeType::Linear {
                    connectors.push(BehaviorNodeConnector::Bottom1);
                    connectors.push(BehaviorNodeConnector::Bottom2);
                    connectors.push(BehaviorNodeConnector::Bottom);
                } else
                if node.behavior_type == BehaviorNodeType::Sequence {
                    connectors.push(BehaviorNodeConnector::Bottom1);
                    connectors.push(BehaviorNodeConnector::Bottom2);
                    connectors.push(BehaviorNodeConnector::Bottom);
                    connectors.push(BehaviorNodeConnector::Bottom3);
                    connectors.push(BehaviorNodeConnector::Bottom4);
                    is_sequence = true;
                } else {
                    if let Some(node_call) = self.nodes.get_mut(&node.behavior_type) {
                        let item_id = item_id;
                        let connector = node_call(instance_index, (item_id, node_id), self, BehaviorType::Items);
                        rc = Some(connector);
                        connectors.push(connector);
                    } else {
                        connectors.push(BehaviorNodeConnector::Bottom);
                    }
                }
            }
        }

        // Search the connections to check if we can find an ongoing node connection
        for connector in connectors {
            if let Some(item) = self.items.get_mut(&item_id) {

                for c in &item.connections {
                    if c.0 == node_id && c.1 == connector {
                        connected_node_ids.push(c.2);
                        if is_sequence == false {
                            self.executed_connections.push((BehaviorType::Items, c.0, c.1));
                        } else {
                            possibly_executed_connections.push((BehaviorType::Items, c.0, c.1));
                        }
                    }
                }
            }
        }

        // And if yes execute it
        for (index, connected_node_id) in connected_node_ids.iter().enumerate() {

            // If this is a sequence, mark this connection as executed
            if is_sequence {
                self.executed_connections.push(possibly_executed_connections[index]);
            }

            if let Some(connector) = self.execute_item_node(instance_index, item_id, *connected_node_id) {
                if is_sequence {
                    // Inside a sequence break out if the connector is not Success
                    if connector == BehaviorNodeConnector::Fail || connector == BehaviorNodeConnector::Right {
                        break;
                    }

                }
            }
        }
        rc
    }

    /// Executes the given spell node and follows the connection chain
    pub fn execute_spell_node(&mut self, instance_index: usize, spell_id: Uuid, node_id: Uuid) -> Option<BehaviorNodeConnector> {

        let mut connectors : Vec<BehaviorNodeConnector> = vec![];
        let mut connected_node_ids : Vec<Uuid> = vec![];
        let mut possibly_executed_connections : Vec<(BehaviorType, Uuid, BehaviorNodeConnector)> = vec![];

        let mut is_sequence = false;
        let mut rc : Option<BehaviorNodeConnector> = None;

        // Call the node and get the resulting BehaviorNodeConnector
        if let Some(item) = self.spells.get_mut(&spell_id) {
            if let Some(node) = item.nodes.get_mut(&node_id) {

                // Handle special nodes
                if node.behavior_type == BehaviorNodeType::BehaviorTree || node.behavior_type == BehaviorNodeType::Linear {
                    connectors.push(BehaviorNodeConnector::Bottom1);
                    connectors.push(BehaviorNodeConnector::Bottom2);
                    connectors.push(BehaviorNodeConnector::Bottom);
                } else
                if node.behavior_type == BehaviorNodeType::Sequence {
                    connectors.push(BehaviorNodeConnector::Bottom1);
                    connectors.push(BehaviorNodeConnector::Bottom2);
                    connectors.push(BehaviorNodeConnector::Bottom);
                    connectors.push(BehaviorNodeConnector::Bottom3);
                    connectors.push(BehaviorNodeConnector::Bottom4);
                    is_sequence = true;
                } else {
                    if let Some(node_call) = self.nodes.get_mut(&node.behavior_type) {
                        let item_id = spell_id;
                        let connector = node_call(instance_index, (item_id, node_id), self, BehaviorType::Spells);
                        rc = Some(connector);
                        connectors.push(connector);
                    } else {
                        connectors.push(BehaviorNodeConnector::Bottom);
                    }
                }
            }
        }

        // Search the connections to check if we can find an ongoing node connection
        for connector in connectors {
            if let Some(item) = self.spells.get_mut(&spell_id) {
                for c in &item.connections {
                    if c.0 == node_id && c.1 == connector {
                        connected_node_ids.push(c.2);
                        if is_sequence == false {
                            self.executed_connections.push((BehaviorType::Spells, c.0, c.1));
                        } else {
                            possibly_executed_connections.push((BehaviorType::Spells, c.0, c.1));
                        }
                    }
                }
            }
        }

        // And if yes execute it
        for (index, connected_node_id) in connected_node_ids.iter().enumerate() {

            // If this is a sequence, mark this connection as executed
            if is_sequence {
                self.executed_connections.push(possibly_executed_connections[index]);
            }

            if let Some(connector) = self.execute_spell_node(instance_index, spell_id, *connected_node_id) {
                if is_sequence {
                    // Inside a sequence break out if the connector is not Success
                    if connector == BehaviorNodeConnector::Fail || connector == BehaviorNodeConnector::Right {
                        break;
                    }

                }
            }
        }
        rc
    }

    /// Executes the given node and follows the connection chain
    pub fn execute_area_node(&mut self, region_id: Uuid, area_index: usize, node_id: Uuid) -> Option<BehaviorNodeConnector> {

        let mut connectors : Vec<BehaviorNodeConnector> = vec![];
        let mut connected_node_ids : Vec<Uuid> = vec![];

        let mut rc : Option<BehaviorNodeConnector> = None;

        // Call the node and get the resulting BehaviorNodeConnector
        if let Some(node) = self.region_behavior[area_index].nodes.get_mut(&node_id) {

            if let Some(node_call) = self.nodes.get_mut(&node.behavior_type) {
                let connector = node_call(area_index, (region_id, node_id), self, BehaviorType::Regions);
                rc = Some(connector);
                connectors.push(connector);
            } else {
                connectors.push(BehaviorNodeConnector::Bottom);
            }
        }

        // Search the connections to check if we can find an ongoing node connection
        for connector in connectors {
            for c in &self.region_behavior[area_index].connections {
                if c.0 == node_id && c.1 == connector {
                    connected_node_ids.push(c.2);
                    self.executed_connections.push((BehaviorType::Regions, c.0, c.1));
                }
            }
        }

        // And if yes execute it
        for (_index, connected_node_id) in connected_node_ids.iter().enumerate() {
            self.execute_area_node(region_id, area_index, *connected_node_id);
        }
        rc
    }

    /// Executes the given node and follows the connection chain
    fn execute_game_node(&mut self, instance_index: usize, node_id: Uuid) -> Option<BehaviorNodeConnector> {

        let mut connectors : Vec<BehaviorNodeConnector> = vec![];
        let mut connected_node_ids : Vec<Uuid> = vec![];
        let mut possibly_executed_connections : Vec<(BehaviorType, Uuid, BehaviorNodeConnector)> = vec![];

        let mut is_sequence = false;
        let mut rc : Option<BehaviorNodeConnector> = None;

        // Call the node and get the resulting BehaviorNodeConnector
        let behavior = &mut self.game_data;
        if let Some(node) = behavior.nodes.get_mut(&node_id) {
            // Handle special nodes
            if node.behavior_type == BehaviorNodeType::Screen{
                connectors.push(BehaviorNodeConnector::Bottom1);
                connectors.push(BehaviorNodeConnector::Bottom2);
                connectors.push(BehaviorNodeConnector::Bottom);
                connectors.push(BehaviorNodeConnector::Bottom3);
                connectors.push(BehaviorNodeConnector::Bottom4);

                if let Some(node_call) = self.nodes.get_mut(&node.behavior_type) {
                    let behavior_id = self.instances[instance_index].behavior_id.clone();
                    _ = node_call(instance_index, (behavior_id, node_id), self, BehaviorType::GameLogic);
                }
            } else
            if node.behavior_type == BehaviorNodeType::BehaviorTree || node.behavior_type == BehaviorNodeType::Linear {
                connectors.push(BehaviorNodeConnector::Bottom1);
                connectors.push(BehaviorNodeConnector::Bottom2);
                connectors.push(BehaviorNodeConnector::Bottom);
                connectors.push(BehaviorNodeConnector::Bottom3);
                connectors.push(BehaviorNodeConnector::Bottom4);
            } else
            if node.behavior_type == BehaviorNodeType::Sequence {
                connectors.push(BehaviorNodeConnector::Bottom1);
                connectors.push(BehaviorNodeConnector::Bottom2);
                connectors.push(BehaviorNodeConnector::Bottom);
                connectors.push(BehaviorNodeConnector::Bottom3);
                connectors.push(BehaviorNodeConnector::Bottom4);
                is_sequence = true;
            } else {
                if let Some(node_call) = self.nodes.get_mut(&node.behavior_type) {
                    let behavior_id = self.instances[instance_index].behavior_id.clone();
                    let connector = node_call(instance_index, (behavior_id, node_id), self, BehaviorType::GameLogic);
                    rc = Some(connector);
                    connectors.push(connector);
                } else {
                    connectors.push(BehaviorNodeConnector::Bottom);
                }
            }
        }

        // Search the connections to check if we can find an ongoing node connection
        for connector in connectors {
            let behavior = &mut self.game_data;
            for c in &behavior.connections {
                if c.0 == node_id && c.1 == connector {
                    connected_node_ids.push(c.2);
                    if is_sequence == false {
                        self.executed_connections.push((BehaviorType::GameLogic, c.0, c.1));
                    } else {
                        possibly_executed_connections.push((BehaviorType::GameLogic, c.0, c.1));
                    }
                }
            }
        }

        // And if yes execute it
        for (index, connected_node_id) in connected_node_ids.iter().enumerate() {

            // If this is a sequence, mark this connection as executed
            if is_sequence {
                self.executed_connections.push(possibly_executed_connections[index]);
            }

            if let Some(connector) = self.execute_game_node(instance_index, *connected_node_id) {
                if is_sequence {
                    // Inside a sequence break out if the connector is not Success
                    if connector == BehaviorNodeConnector::Fail || connector == BehaviorNodeConnector::Right {
                        break;
                    }
                }
            }
        }
        rc
    }

    /// Setup the region instance data by decoding the JSON for all game elements and sets up the npc and game behavior instances.
    pub fn setup(&mut self, region: String, region_behavior: FxHashMap<Uuid, Vec<String>>, behaviors: Vec<String>, systems: Vec<String>, items: Vec<String>, spells: Vec<String>, game: String, scripts: FxHashMap<String, String>) {
        // Decode all JSON
        if let Some(region_data) = serde_json::from_str::<GameRegionData>(&region).ok() {

            let data: &mut RegionData = &mut REGION_DATA.borrow_mut()[*CURR_INST.borrow()];
            data.region_data = region_data.clone();
            self.region_data = region_data;


            if let Some(property) = data.region_data.settings.get("movement") {
                if let Some(value) = property.as_string() {
                    if value.to_lowercase() == "tile" {
                        data.pixel_based_movement = false;
                    }
                }
            }

            if let Some(areas) = region_behavior.get(&data.region_data.id) {
                for a in areas {
                    if let Some(ab) = serde_json::from_str::<GameBehaviorData>(&a).ok() {
                        data.region_area_behavior.push(ab);
                    }
                }
            }
        }
        for b in behaviors {
            if let Some(behavior_data) = serde_json::from_str::<GameBehaviorData>(&b).ok() {
                self.behaviors.insert(behavior_data.id, behavior_data);
            }
        }
        for s in systems {
            if let Some(behavior_data) = serde_json::from_str::<GameBehaviorData>(&s).ok() {
                self.system_names.push(behavior_data.name.clone());
                self.system_ids.push(behavior_data.id.clone());
                self.systems.insert(behavior_data.id, behavior_data);
            }
        }
        for i in items {
            if let Some(behavior_data) = serde_json::from_str::<GameBehaviorData>(&i).ok() {
                if let Some(instances) = &behavior_data.loot {
                    for instance in instances {
                        if instance.position.region != self.region_data.id { continue; }
                        let mut loot = Item::new(behavior_data.id, behavior_data.name.clone());
                        loot.item_type = "gear".to_string();
                        loot.amount = instance.amount;
                        loot.stackable = 1;

                        for (_index, node) in &behavior_data.nodes {
                            if node.behavior_type == BehaviorNodeType::BehaviorType {
                                if let Some(value) = node.values.get(&"tile".to_string()) {
                                    loot.tile = value.to_tile_data();
                                }
                                if let Some(value) = node.values.get(&"settings".to_string()) {
                                    if let Some(str) = value.to_string() {
                                        let mut s = PropertySink::new();
                                        s.load_from_string(str.clone());
                                        loot.read_from_sink(&s);
                                    }
                                }
                            }
                        }

                        let data = &mut REGION_DATA.borrow_mut()[*CURR_INST.borrow()];
                        if let Some(v) = data.loot.get_mut(&(instance.position.x, instance.position.y)) {
                            v.push(loot);
                        } else {
                            data.loot.insert((instance.position.x, instance.position.y), vec![loot]);
                        }
                    }
                }
                self.items.insert(behavior_data.id, behavior_data);
            }
        }
        for i in spells {
            if let Some(behavior_data) = serde_json::from_str::<GameBehaviorData>(&i).ok() {
                self.spells.insert(behavior_data.id, behavior_data);
            }
        }
        if let Some(game_data) = serde_json::from_str(&game).ok() {
            self.game_data = game_data;

            // Update the game settings, just in case they don't contain the latest
            if self.game_data.settings.is_some() {
                crate::gamedata::game::update_game_sink(&mut self.game_data.settings.as_mut().unwrap());
            }

            // Read global game settings

            if let Some(settings) = &self.game_data.settings {
                if let Some(screen_size) = settings.get("screen_size") {
                    match screen_size.value {
                        PropertyValue::IntX(v) => {
                            self.screen_size = (v[0], v[1]);
                        },
                        _ => {}
                    }
                }
                if let Some(def_square_tile_size) = settings.get("def_square_tile_size") {
                    match def_square_tile_size.value {
                        PropertyValue::Int(v) => {
                            self.def_square_tile_size = v;
                        },
                        _ => {}
                    }
                }

                if let Some(property) = settings.get("gear_slots") {
                    if let Some(name) = property.as_string() {
                        let ar : Vec<&str> = name.split(",").collect();
                        for s in ar {
                            self.gear_slots.push(s.to_lowercase().trim().to_string());
                        }
                    }
                }

                if let Some(property) = settings.get("weapon_slots") {
                    if let Some(name) = property.as_string() {
                        let ar : Vec<&str> = name.split(",").collect();
                        for s in ar {
                            self.weapon_slots.push(s.to_lowercase().trim().to_string());
                        }
                    }
                }

                if let Some(property) = settings.get("ticks_per_minute") {
                    if let Some(ticks) = property.as_int() {
                        self.ticks_per_minute = ticks as usize;
                        *TICKS_PER_MINUTE.borrow_mut() = ticks as usize;
                    }
                }
            }
        }

        self.scripts = scripts;

        // Create all behavior instances of characters inside this region
        let ids : Vec<Uuid> = self.behaviors.keys().cloned().collect();
        for id in ids {
            self.create_behavior_instance(id, true, None);
        }

        // Create the game instance itself
        let mut to_execute : Vec<Uuid> = vec![];
        let mut startup_name : Option<String> = None;
        let mut locked_tree  : Option<Uuid> = None;
        let scope = rhai::Scope::new();
        let behavior = &mut self.game_data;

        // Collect name of the startup tree and the variables
        for (_id, node) in &behavior.nodes {
            if node.behavior_type == BehaviorNodeType::BehaviorType {
                if let Some(value )= node.values.get(&"startup".to_string()) {
                    startup_name = Some(value.to_string_value());
                }
            }
        }

        // Second pass parse the trees and find the startup tree
        for (id, node) in &behavior.nodes {
            if node.behavior_type == BehaviorNodeType::BehaviorTree {
                for c in &behavior.connections {
                    if c.0 == *id {
                        to_execute.push(c.0);
                        if let Some(startup) = startup_name.clone() {
                            if node.name == startup {
                                locked_tree = Some(node.id);
                            }
                        }
                    }
                }
            }
        }

        let index = self.instances.len();

        let instance = BehaviorInstance {id: Uuid::new_v4(), state: BehaviorInstanceState::Normal, name: behavior.name.clone(), behavior_id: behavior.id, tree_ids: to_execute.clone(), position: None, tile: None, target_instance_index: None, locked_tree, party: vec![], node_values: FxHashMap::default(), scope_buffer: None, sleep_cycles: 0, systems_id: Uuid::new_v4(), action: None, instance_type: BehaviorInstanceType::GameLogic, update: None, regions_send: std::collections::HashSet::new(), curr_player_screen_id: None, game_locked_tree: None, curr_player_screen: "".to_string(), curr_player_widgets: vec![], messages: vec![], audio: vec![], old_position: None, max_transition_time: 0, curr_transition_time: 0, alignment: 1, multi_choice_data: vec![], communication: vec![], multi_choice_answer: None, damage_to_be_dealt: None, inventory_buffer: None, weapons_buffer: None, gear_buffer: None, skills_buffer: None, experience_buffer: None, effects: vec![], healing_to_be_dealt: None, instance_creation_data: None, send_screen_scripts: false };

        self.instances.push(instance.clone());
        self.scopes.push(scope);

        {
            let data = &mut REGION_DATA.borrow_mut()[*CURR_INST.borrow()];
            data.sheets.push(Sheet::new());
            data.character_instances.push(instance);
        }

        for tree_id in &to_execute {
            // Execute this tree if it is a "Startup" Only tree
            if let Some(value)= self.get_game_node_value(*tree_id, "execute") {
                if let Some(value) = value.to_integer() {
                    if value == 1 {
                        self.execute_game_node(index, tree_id.clone());
                    }
                }
            }
        }
        self.game_instance_index = Some(index);

        let mut loot_map;

        {
            let data = &mut REGION_DATA.borrow_mut()[*CURR_INST.borrow()];
            data.game_instance_index = Some(index);
            loot_map = data.loot.clone();
        }

        // We iterate over all loot and initialize state if necessary

        for (pos, loot) in &mut loot_map {
            for index in 0..loot.len() {
                if let Some(mut state) = check_and_create_item_state(loot[index].id) {
                    if let Some(light) = &mut state.light {
                        light.position = pos.clone();
                    }
                    loot[index].state = Some(state);
                }
            }
        }

        let data = &mut REGION_DATA.borrow_mut()[*CURR_INST.borrow()];
        data.loot = loot_map;
    }

    /// Creates a new player instance
    pub fn create_player_instance(&mut self, uuid: Uuid, position: Position) {
        let mut player_id : Option<Uuid> = None;
        for b in &self.behaviors {
            if b.1.name == "Player" {
                player_id = Some(*b.0);
            }
        }
        if let Some(player_id) = player_id {
            let index = self.create_behavior_instance(player_id, false, None);
            self.instances[index].instance_type = BehaviorInstanceType::Player;
            self.instances[index].id = uuid;
            self.instances[index].position = Some(position.clone());
            let data: &mut RegionData = &mut REGION_DATA.borrow_mut()[*CURR_INST.borrow()];
            data.character_instances[index].instance_type = BehaviorInstanceType::Player;
            data.character_instances[index].id = uuid;
            data.character_instances[index].position = Some(position);
            data.player_uuid_indices.insert(uuid, index);
            log::info!("Player instance {} created.", uuid);
        }
    }

    /// Destroyes a player instance
    pub fn destroy_player_instance(&mut self, uuid: Uuid) {
        for inst_index in 0..self.instances.len() {
            if self.instances[inst_index].id == uuid {
                self.purge_instance(inst_index);
                break;
            }
        }
    }

    /// Creates an instance of a behavior (character)
    fn create_behavior_instance(&mut self, id: Uuid, npc_only: bool, data: Option<CharacterInstanceData>) -> usize {

        let mut index = 0;

        let mut startup_trees               : Vec<Uuid> = vec![];
        let mut behavior_name       = "".to_string();
        let mut behavior_id           = Uuid::new_v4();
        let mut class_name                  : Option<String> = None;
        let mut race_name                   : Option<String> = None;

        let mut to_create : Vec<CharacterInstanceData> = vec![];

        // Collect all the default data for the behavior from the nodes: Position, tile, behavior Trees and variables.
        let mut to_execute              : Vec<Uuid> = vec![];
        let mut default_position        : Option<Position> = None;
        let mut default_tile            : Option<TileId> = None;
        let mut default_alignment       : i32 = 1;
        let mut settings_sink= PropertySink::new();
        let default_scope     = rhai::Scope::new();

        // Instances to create for this behavior
        if let Some(behavior) = &self.behaviors.get_mut(&id) {

            behavior_name = behavior.name.clone();
            behavior_id = behavior.id.clone();

            if npc_only && behavior.name == "Player" {
                return index;
            }

            for (id, node) in &behavior.nodes {
                if node.behavior_type == BehaviorNodeType::BehaviorTree {
                    for (value_name, value) in &node.values {
                        if *value_name == "execute".to_string() {
                            if let Some(v) = value.to_integer() {
                                if v == 1 {
                                    // Startup only tree
                                    for c in &behavior.connections {
                                        if c.0 == *id {
                                            startup_trees.push(c.0);
                                        }
                                    }
                                    break;
                                } else
                                if v == 0 {
                                    // Always
                                    for c in &behavior.connections {
                                        if c.0 == *id {
                                            to_execute.push(c.0);
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else
                if node.behavior_type == BehaviorNodeType::BehaviorType {
                    if let Some(value )= node.values.get(&"position".to_string()) {
                        default_position = value.to_position();
                    }
                    if let Some(value )= node.values.get(&"tile".to_string()) {
                        default_tile = value.to_tile_id()
                    }
                    if let Some(value )= node.values.get(&"settings".to_string()) {
                        if let Some(settings) = value.to_string() {
                            settings_sink.load_from_string(settings);
                        }
                    }
                    if let Some(value )= node.values.get(&"alignment".to_string()) {
                        if let Some(alignment) = value.to_integer() {
                            default_alignment = 2 - alignment- 1;
                        }
                    }
                }
            }
            // Add main
            if default_position.is_some() && default_tile.is_some() && data.is_none() {
                let main = CharacterInstanceData {
                    position    : default_position.unwrap().clone(),
                    tile        : default_tile.clone(),
                    name        : Some(behavior.name.clone()),
                    alignment   : default_alignment
                };
                to_create.push(main)
            }
            if data.is_none() {
            // Add the instances of main
                if let Some(instances) = &behavior.instances {
                    for i in instances {
                        let mut inst = (*i).clone();
                        if inst.name.is_none() {
                            inst.name = Some(behavior.name.clone());
                        }
                        if inst.tile.is_none() {
                            inst.tile = default_tile.clone();
                        }
                        to_create.push(inst);
                    }
                }
            } else {
                // If we get the character instance data, only add this (respawn)
                to_create.push(data.unwrap());
            }
        }

        // Now we have all instances of the behavior we need to create
        for inst in to_create {

            // Only create when instance is in this region
            if inst.position.region != self.region_data.id {
                continue;
            }

            //println!("Creating instance {}", inst.name.unwrap());

            let instance: BehaviorInstance = BehaviorInstance {id: uuid::Uuid::new_v4(), state: BehaviorInstanceState::Normal, name: behavior_name.clone(), behavior_id: behavior_id, tree_ids: to_execute.clone(), position: Some(inst.position.clone()), tile: inst.tile.clone(), target_instance_index: None, locked_tree: None, party: vec![], node_values: FxHashMap::default(), scope_buffer: None, sleep_cycles: 0, systems_id: Uuid::new_v4(), action: None, instance_type: BehaviorInstanceType::NonPlayerCharacter, update: None, regions_send: std::collections::HashSet::new(), curr_player_screen_id: None, game_locked_tree: None, curr_player_screen: "".to_string(), curr_player_widgets: vec![], messages: vec![], audio: vec![], old_position: None, max_transition_time: 0, curr_transition_time: 0, alignment: inst.alignment, multi_choice_data: vec![], communication: vec![], multi_choice_answer: None, damage_to_be_dealt: None, inventory_buffer: None, weapons_buffer: None, gear_buffer: None, skills_buffer: None, experience_buffer: None, effects: vec![], healing_to_be_dealt: None, instance_creation_data: Some(inst.clone()), send_screen_scripts: false };

            index = self.instances.len();
            self.instances.push(instance.clone());

            // Create skills

            let mut skills = Skills::new();

            for (_id, behavior) in &self.systems {
                if behavior.name.to_lowercase() == "skills" {
                    for (_id, node) in &behavior.nodes {
                        if node.behavior_type == BehaviorNodeType::SkillTree {
                            skills.add_skill(node.name.clone());

                            // Add the skill to the skill_tree

                            let mut rc : Vec<(i32, String, String)> = vec![];
                            let mut parent_id = node.id;

                            loop {
                                let mut found = false;
                                for (id1, c1, id2, c2) in &behavior.connections {
                                    if *id1 == parent_id && *c1 == BehaviorNodeConnector::Bottom {
                                        for (uuid, node) in &behavior.nodes {
                                            if *uuid == *id2 {
                                                let mut start = 0;
                                                if let Some(value) = node.values.get(&"start".to_string()) {
                                                    if let Some(i) = value.to_integer() {
                                                        start = i;
                                                    }
                                                }
                                                let mut message = "".to_string();
                                                if let Some(value) = node.values.get(&"message".to_string()) {
                                                    if let Some(m) = value.to_string() {
                                                        message = m;
                                                    }
                                                }

                                                parent_id = node.id;
                                                found = true;

                                                rc.push((start, node.name.clone(), message));
                                            }
                                        }
                                    } else
                                    if *id2 == parent_id && *c2 == BehaviorNodeConnector::Bottom {
                                        for (uuid, node) in &behavior.nodes {
                                            if *uuid == *id1 {
                                                let mut start = 0;
                                                if let Some(value) = node.values.get(&"start".to_string()) {
                                                    if let Some(i) = value.to_integer() {
                                                        start = i;
                                                    }
                                                }
                                                let mut message = "".to_string();
                                                if let Some(value) = node.values.get(&"message".to_string()) {
                                                    if let Some(m) = value.to_string() {
                                                        message = m;
                                                    }
                                                }
                                                parent_id = node.id;
                                                found = true;

                                                rc.push((start, node.name.clone(), message));
                                            }
                                        }
                                    }
                                }
                                if found == false {
                                    break;
                                }
                            }

                            self.skill_trees.insert(node.name.clone(), rc);
                        }
                    }
                }
            }

            // println!("{:?}", self.skill_trees);

            // Set the default values into the scope
            let mut scope = default_scope.clone();
            scope.set_value("name", behavior_name.clone());
            scope.set_value("alignment", inst.alignment as i32);
            scope.set_value("message", ScriptMessageCmd::new());
            scope.set_value("skills", skills);
            scope.set_value("experience", Experience::new());
            scope.set_value("date", DATE.borrow().clone());
            scope.set_value("failure", FailureEnum::No);

            let mut system_startup_trees : Vec<String> = vec![];

            if let Some(class) = settings_sink.get("class") {
                if let Some(cl) = class.as_string() {
                    class_name = Some(cl.clone());
                    system_startup_trees.push(cl);
                }
            }
            if let Some(race) = settings_sink.get("race") {
                if let Some(ra) = race.as_string() {
                    race_name = Some(ra.clone());
                    system_startup_trees.push(ra);
                }
            }

            let mut startup_system_trees        : Vec<(Uuid, Uuid)> = vec![];

            // Execute the startup trees in the given systems for execution (for class and race)
            for system_name in system_startup_trees {
                if self.system_names.contains(&system_name) {
                    for (system_id, system) in &self.systems {
                        if system.name == system_name {
                            for (id, node) in &system.nodes {
                                if node.behavior_type == BehaviorNodeType::BehaviorTree {
                                    for (value_name, value) in &node.values {
                                        if *value_name == "execute".to_string() {
                                            if let Some(v) = value.to_integer() {
                                                if v == 1 {
                                                    // Startup only tree
                                                    for c in &system.connections {
                                                        if c.0 == *id {
                                                            startup_system_trees.push((*system_id, c.0));
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Add Spells appropriate for this character

            let mut spells = Spells::new();

            for (id, behavior) in &self.spells {
                for (_id, node) in &behavior.nodes {
                    if node.behavior_type == BehaviorNodeType::BehaviorType {
                        if let Some(value )= node.values.get(&"settings".to_string()) {

                            let mut spell_tile            : Option<TileData> = None;
                            let mut spells_sink= PropertySink::new();
                            let mut spell_distance = 3;

                            if let Some(value )= node.values.get(&"tile".to_string()) {
                                spell_tile = value.to_tile_data();
                            }
                            if let Some(settings) = value.to_string() {
                                spells_sink.load_from_string(settings);
                            }

                            let mut include_spell = false;

                            if let Some(c) = spells_sink.get_as_string_array("classes") {
                                if c[0].to_lowercase() == "all" {
                                    include_spell = true;
                                } else
                                if let Some(class_name) = &class_name {
                                    for v in 0..c.len() {
                                        if c[v].to_lowercase() == class_name.to_lowercase() {
                                            include_spell = true;
                                            break;
                                        }
                                    }
                                }
                            }
                            if let Some(distance) = spells_sink.get(&"spell_distance") {
                                if let Some(d)  = distance.as_int() {
                                    spell_distance = d;
                                }
                            }

                            if include_spell {
                                let mut spell = Spell::new(*id, behavior.name.to_string());
                                spell.tile = spell_tile;
                                spell.distance = spell_distance;
                                spells.spells.push(spell);

                            }
                        }
                        break;
                    }
                }
            }

            // --- End Spells

            // Set the sheet
            {
                let data = &mut REGION_DATA.borrow_mut()[*CURR_INST.borrow()];
                let mut sheet = Sheet::new();
                sheet.name = behavior_name.clone();
                if let Some(class_name) = class_name.clone() {
                    sheet.class = class_name;
                }
                if let Some(race_name) = race_name.clone() {
                    sheet.race = race_name;
                }
                sheet.spells = spells;
                data.sheets.push(sheet);
                data.character_instances.push(instance);
                data.curr_index = index;
            }


            self.scopes.push(scope);

            if index < self.instances.len() {

                // Set the class based level tree
                if let Some(class_name) = class_name.clone() {
                    self.set_level_tree(index, class_name);
                }

                // Execute the system startup trees
                for startup_id in &startup_system_trees {
                    self.instances[index].systems_id = startup_id.0;
                    self.execute_systems_node(index, startup_id.1);
                }

                // Execute the startup only trees
                for startup_id in &startup_trees {
                    {
                        REGION_DATA.borrow_mut()[*CURR_INST.borrow()].curr_index = index;
                    }
                    execute_node(behavior_id, startup_id.clone(), &mut BEHAVIORS.borrow_mut());
                }
            }
        }

        index
    }

    /// Returns a game node value
    fn get_game_node_value(&mut self, node_id: Uuid, node_property: &str) -> Option<Value> {
        if let Some(node) = self.game_data.nodes.get(&node_id) {
            if let Some(value) = node.values.get(node_property) {
                return Some(value.clone());
            }
        }
        None
    }

    /// Returns the layered tiles at the given position and checks for displacements
    pub fn get_tile_at(&self, pos: (isize, isize)) -> Vec<TileData> {
        let mut rc = vec![];
        if let Some(t) = self.displacements.get(&pos) {
            rc.push(t.clone());
        } else {
            if let Some(t) = self.region_data.layer1.get(&pos) {
                rc.push(t.clone());
            }
            if let Some(t) = self.region_data.layer2.get(&pos) {
                rc.push(t.clone());
            }
            if let Some(t) = self.region_data.layer3.get(&pos) {
                rc.push(t.clone());
            }
            if let Some(t) = self.region_data.layer4.get(&pos) {
                rc.push(t.clone());
            }
        }
        rc
    }

    /// Returns the layered tiles at the given position and checks for displacements
    pub fn get_tile_without_displacements_at(&self, pos: (isize, isize)) -> Vec<TileData> {
        let mut rc = vec![];

        if let Some(t) = self.region_data.layer1.get(&pos) {
            rc.push(t.clone());
        }
        if let Some(t) = self.region_data.layer2.get(&pos) {
            rc.push(t.clone());
        }
        if let Some(t) = self.region_data.layer3.get(&pos) {
            rc.push(t.clone());
        }
        if let Some(t) = self.region_data.layer4.get(&pos) {
            rc.push(t.clone());
        }
        rc
    }

    /// Gets the behavior for the given id
    pub fn get_behavior(&self, id: Uuid, behavior_type: BehaviorType) -> Option<&GameBehaviorData> {
        if behavior_type == BehaviorType::Regions {
            for b in &self.region_behavior {
                if b.id == id {
                    return Some(&b);
                }
            }
        } else
        if behavior_type == BehaviorType::Behaviors {
            return self.behaviors.get(&id);
        } else
        if behavior_type == BehaviorType::Systems {
            return self.systems.get(&id);
        } else
        if behavior_type == BehaviorType::Items {
            return self.items.get(&id);
        } else
        if behavior_type == BehaviorType::GameLogic {
            return Some(&self.game_data);
        }
        None
    }

    /// Gets the mutable behavior for the given behavior type
    pub fn get_mut_behavior(&mut self, id: Uuid, behavior_type: BehaviorType) -> Option<&mut GameBehaviorData> {
        if behavior_type == BehaviorType::Regions {
            for b in &mut self.region_behavior {
                if b.id == id {
                    return Some(b);
                }
            }
        } else
        if behavior_type == BehaviorType::Behaviors {
            return self.behaviors.get_mut(&id);
        } else
        if behavior_type == BehaviorType::Systems {
            return self.systems.get_mut(&id);
        } else
        if behavior_type == BehaviorType::Items {
            return self.items.get_mut(&id);
        } else
        if behavior_type == BehaviorType::GameLogic {
            return Some(&mut self.game_data);
        }
        None
    }

    /// Purges this instance, voiding it.
    pub fn purge_instance(&mut self, inst_index: usize) {
        let data: &mut RegionData = &mut REGION_DATA.borrow_mut()[*CURR_INST.borrow()];
        data.character_instances[inst_index].state = BehaviorInstanceState::Purged;
        data.player_uuid_indices.remove(&data.character_instances[inst_index].id);
    }

    /// Transfers a character instance into this region
    pub fn transfer_character_into(&mut self, mut instance: BehaviorInstance, sheet: Sheet) {
        // TODO, fill in purged
        let data: &mut RegionData = &mut REGION_DATA.borrow_mut()[*CURR_INST.borrow()];
        data.player_uuid_indices.insert(instance.id, data.character_instances.len());

        let mut scope = rhai::Scope::new();
        self.deserialize_character_instance(&mut instance, &mut scope);

        self.instances.push(instance.clone());
        data.character_instances.push(instance);
        data.sheets.push(sheet);
        self.scopes.push(scope);
    }

    /// Sets the debugging behavior id.
    pub fn set_debug_behavior_id(&mut self, behavior_id: Uuid) {
        self.debug_behavior_id = Some(behavior_id);
    }

    /// Serializes the given instance index
    fn serialize_character_instance(&mut self, inst_index: usize) {
        // Serialize character
        let mut scope_buffer = ScopeBuffer::new();
        scope_buffer.read_from_scope(&self.scopes[inst_index]);

        if let Some(mess) = self.scopes[inst_index].get_mut("inventory") {
            if let Some(inv) = mess.write_lock::<Inventory>() {

                let i = inv.clone();
                if let Some(json) = serde_json::to_string(&i).ok() {
                        self.instances[inst_index].inventory_buffer = Some(json);
                }
            }
        }

        if let Some(mess) = self.scopes[inst_index].get_mut("weapons") {
            if let Some(weap) = mess.write_lock::<Weapons>() {

                let w = weap.clone();
                if let Some(json) = serde_json::to_string(&w).ok() {
                        self.instances[inst_index].weapons_buffer = Some(json);
                }
            }
        }

        if let Some(mess) = self.scopes[inst_index].get_mut("gear") {
            if let Some(ge) = mess.write_lock::<Gear>() {

                let g = ge.clone();
                if let Some(json) = serde_json::to_string(&g).ok() {
                        self.instances[inst_index].gear_buffer = Some(json);
                }
            }
        }


        if let Some(mess) = self.scopes[inst_index].get_mut("skills") {
            if let Some(sk) = mess.write_lock::<Skills>() {

                let s = sk.clone();
                if let Some(json) = serde_json::to_string(&s).ok() {
                        self.instances[inst_index].skills_buffer = Some(json);
                }
            }
        }

        if let Some(mess) = self.scopes[inst_index].get_mut("experience") {
            if let Some(ex) = mess.write_lock::<Experience>() {

                let e = ex.clone();
                if let Some(json) = serde_json::to_string(&e).ok() {
                        self.instances[inst_index].experience_buffer = Some(json);
                }
            }
        }

        self.instances[inst_index].scope_buffer = Some(scope_buffer);
    }

    /// Deserializes the given instance
    fn deserialize_character_instance(&self, instance: &mut BehaviorInstance, mut scope: &mut Scope) {
        if let Some(buffer) = &instance.scope_buffer {
            buffer.write_to_scope(&mut scope);
        }

        scope.set_value("message", ScriptMessageCmd::new());

        if let Some(inventory_buffer) = &instance.inventory_buffer {
            let inventory : Inventory = serde_json::from_str(&inventory_buffer)
                .unwrap_or(Inventory::new());
            scope.set_value("inventory", inventory);
        } else {
            // Should not happen
            scope.set_value("inventory", Inventory::new());
        }

        if let Some(weapons_buffer) = &instance.weapons_buffer {
            let weapons : Weapons = serde_json::from_str(&weapons_buffer)
                .unwrap_or(Weapons::new());
            scope.set_value("weapons", weapons);
        } else {
            // Should not happen
            scope.set_value("weapons", Weapons::new());
        }

        if let Some(gear_buffer) = &instance.gear_buffer {
            let gear : Gear = serde_json::from_str(&gear_buffer)
                .unwrap_or(Gear::new());
            scope.set_value("gear", gear);
        } else {
            // Should not happen
            scope.set_value("gear", Gear::new());
        }

        if let Some(skills_buffer) = &instance.skills_buffer {
            let skills : Skills = serde_json::from_str(&skills_buffer)
                .unwrap_or(Skills::new());
            scope.set_value("skills", skills);
        } else {
            // Should not happen
            scope.set_value("skills", Skills::new());
        }

        if let Some(experience_buffer) = &instance.experience_buffer {
            let experience : Experience = serde_json::from_str(&experience_buffer)
                .unwrap_or(Experience::new());
            scope.set_value("experience", experience);
        } else {
            // Should not happen
            scope.set_value("experience", Experience::new());
        }

    }

    /// Gets the current time in milliseconds
    pub fn get_time(&self) -> u128 {
        #[cfg(target_arch = "wasm32")]
        {
            web_sys::window().unwrap().performance().unwrap().now() as u128
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let stop = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("Time went backwards");
                stop.as_millis()
        }
    }

    /// Set class based level tree for a character
    pub fn set_level_tree(&mut self, instance_index: usize, system_name: String) {

        let tree_name = "Level Tree".to_string();

        let mut levels : Vec<(i32, String, Uuid)> = vec![];
        let mut level_tree_id = Uuid::new_v4();
        let mut experience_msg : String = "You gained {} experience.".to_string();

        for (_id, behavior) in &self.systems {
            if behavior.name == system_name {
                for (_id, node) in &behavior.nodes {
                    if node.name == tree_name {

                        if let Some(value) = node.values.get(&"message".to_string()) {
                            if let Some(m) = value.to_string() {
                                experience_msg = m;
                            }
                        }
                        // Store the levels

                        let mut rc : Vec<(i32, String, Uuid)> = vec![];
                        let mut parent_id = node.id;

                        level_tree_id = node.id;

                        loop {
                            let mut found = false;
                            for (id1, c1, id2, c2) in &behavior.connections {
                                if *id1 == parent_id && *c1 == BehaviorNodeConnector::Bottom {
                                    for (uuid, node) in &behavior.nodes {
                                        if *uuid == *id2 {
                                            let mut start = 0;
                                            if let Some(value) = node.values.get(&"start".to_string()) {
                                                if let Some(i) = value.to_integer() {
                                                    start = i;
                                                }
                                            }
                                            let mut message = "".to_string();
                                            if let Some(value) = node.values.get(&"message".to_string()) {
                                                if let Some(m) = value.to_string() {
                                                    message = m;
                                                }
                                            }

                                            parent_id = node.id;
                                            found = true;

                                            rc.push((start, message, parent_id));
                                        }
                                    }
                                } else
                                if *id2 == parent_id && *c2 == BehaviorNodeConnector::Bottom {
                                    for (uuid, node) in &behavior.nodes {
                                        if *uuid == *id1 {
                                            let mut start = 0;
                                            if let Some(value) = node.values.get(&"start".to_string()) {
                                                if let Some(i) = value.to_integer() {
                                                    start = i;
                                                }
                                            }
                                            let mut message = "".to_string();
                                            if let Some(value) = node.values.get(&"message".to_string()) {
                                                if let Some(m) = value.to_string() {
                                                    message = m;
                                                }
                                            }
                                            parent_id = node.id;
                                            found = true;

                                            rc.push((start, message, parent_id));
                                        }
                                    }
                                }
                            }
                            if found == false {
                                break;
                            }
                        }

                        levels = rc;
                    }
                }
            }
        }

        if let Some(e) = self.scopes[instance_index].get_mut("experience") {
            if let Some(mut exp) = e.write_lock::<Experience>() {
                exp.system_name = Some(system_name);
                exp.tree_name = Some(tree_name.to_string());
                exp.levels = levels;
                exp.experience_msg = experience_msg;
                exp.level_tree_id = level_tree_id;
            }
        }

    }

}
