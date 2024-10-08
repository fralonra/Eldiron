use crate::prelude::*;

pub mod io;
pub mod lobby;
pub mod message;
pub mod nodes;
pub mod region_data;
pub mod region_instance;
pub mod region_pool;
pub mod region_utlity;
pub mod script_utilities;
pub mod sheet_utilities;
pub mod user;

use crossbeam_channel::{unbounded, Receiver, Sender};

pub struct RegionPoolMeta {
    sender: Sender<Message>,
    _receiver: Receiver<Message>,

    region_ids: Vec<Uuid>,
}

pub struct Server {
    to_server_receiver: Receiver<Message>,
    to_server_sender: Sender<Message>,

    pub regions: FxHashMap<Uuid, String>,
    pub region_behavior: FxHashMap<Uuid, Vec<String>>,

    pub behavior: Vec<String>,
    pub systems: Vec<String>,
    pub items: Vec<String>,
    pub spells: Vec<String>,
    pub game: String,
    pub scripts: FxHashMap<String, String>,

    /// If we don't use threads (for example for the web), all regions are in here.
    pub pool: Option<RegionPool>,

    /// If we don't use threads (for example for the web), the player lobby is here.
    lobby: Option<Lobby>,
    lobby_sender: Option<Sender<Message>>,

    /// The meta data for all pools
    metas: Vec<RegionPoolMeta>,

    /// The default starting position for players
    player_default_position: Option<Position>,

    /// The region ids for each player uuid so that we know where to send messages.
    players_region_ids: HashMap<Uuid, Uuid>,

    /// We are multi-threaded
    threaded: bool,

    server_io: Option<Box<dyn ServerIO>>,

    // Lookup table for users who have a valid username
    user_names: FxHashMap<Uuid, String>,

    // Allow local users, disable for server based games
    pub allow_local_users: bool,
}

impl Server {
    pub fn new() -> Self {
        let (sender, receiver) = unbounded();

        Self {
            to_server_receiver: receiver,
            to_server_sender: sender,

            lobby: None,
            lobby_sender: None,

            regions: FxHashMap::default(),
            region_behavior: FxHashMap::default(),

            behavior: vec![],
            systems: vec![],
            items: vec![],
            spells: vec![],
            game: "".to_string(),
            pool: None,
            metas: vec![],

            scripts: FxHashMap::default(),

            player_default_position: None,
            players_region_ids: HashMap::new(),

            threaded: false,

            server_io: None,

            user_names: FxHashMap::default(),

            allow_local_users: true,
        }
    }

    /// Set the server io class.
    pub fn set_io(&mut self, io: Box<dyn ServerIO>) {
        self.server_io = Some(io);
    }

    /// Collects all data (assets, regions, behaviors etc.) and store them as JSON so that we can distribute them to threads as needed.
    pub fn collect_data(&mut self, data: &GameData) {
        for (id, region) in &data.regions {
            if let Some(json) = serde_json::to_string(&region.data).ok() {
                self.regions.insert(*id, json);

                let mut behavior = vec![];
                for b in &region.behaviors {
                    if let Some(json) = serde_json::to_string(&b.data).ok() {
                        behavior.push(json);
                    }
                }
                self.region_behavior.insert(*id, behavior);
            }
        }

        for (id, behavior) in &data.behaviors {
            if behavior.data.name == "Player" {
                self.player_default_position = data.get_behavior_default_position(*id);
            }
            if let Some(json) = serde_json::to_string(&behavior.data).ok() {
                self.behavior.push(json);
            }
        }

        for (_id, system) in &data.systems {
            if let Some(json) = serde_json::to_string(&system.data).ok() {
                self.systems.push(json);
            }
        }

        for (_id, item) in &data.items {
            if let Some(json) = serde_json::to_string(&item.data).ok() {
                self.items.push(json);
            }
        }

        for (_id, spell) in &data.spells {
            if let Some(json) = serde_json::to_string(&spell.data).ok() {
                self.spells.push(json);
            }
        }

        if let Some(json) = serde_json::to_string(&data.game.behavior.data).ok() {
            self.game = json;
        }

        self.scripts = data.scripts.clone();
    }

    /// Starts the server and distributes regions over threads. max_num_threads limits the max number of threads or does not use threads at all if None.
    pub fn start(&mut self, max_num_threads: Option<i32>) -> Result<(), String> {
        if let Some(_max_num_threads) = max_num_threads {
            self.threaded = true;

            let max_regions_per_pool = 100;

            let mut regions = vec![];
            let mut region_ids: Vec<Uuid> = vec![];

            let mut start_thread = |region_ids: Vec<Uuid>, regions: Vec<String>| {
                let (sender, receiver) = unbounded();

                let r = receiver.clone();

                let mut region_behavior: FxHashMap<Uuid, Vec<String>> = FxHashMap::default();
                for rid in &region_ids {
                    let behavior = self.region_behavior.get(rid).unwrap().clone();
                    region_behavior.insert(*rid, behavior);
                }

                let meta = RegionPoolMeta {
                    sender,
                    _receiver: receiver,
                    region_ids,
                };

                let behaviors = self.behavior.clone();
                let systems = self.systems.clone();
                let items = self.items.clone();
                let spells: Vec<String> = self.spells.clone();
                let game = self.game.clone();
                let scripts = self.scripts.clone();

                let to_server_sender = self.to_server_sender.clone();

                let _handle = std::thread::spawn(move || {
                    let mut pool = RegionPool::new(true, to_server_sender, r);
                    pool.add_regions(
                        regions,
                        region_behavior,
                        behaviors,
                        systems,
                        items,
                        spells,
                        game,
                        scripts,
                    );
                });

                //meta.sender.send(Message::Status("Startup".to_string())).unwrap();
                self.metas.push(meta);
            };

            for (id, json) in &self.regions {
                if regions.len() < max_regions_per_pool {
                    regions.push(json.clone());
                    region_ids.push(*id);
                } else {
                    start_thread(region_ids, regions.clone());
                    regions = vec![];
                    region_ids = vec![];
                }
            }

            if regions.is_empty() == false {
                start_thread(region_ids, regions.clone());
            }

            // Start the lobby thread

            let (sender, receiver) = unbounded();
            let to_server_sender = self.to_server_sender.clone();

            let game = self.game.clone();
            let scripts = self.scripts.clone();

            let _handle = std::thread::spawn(move || {
                let mut lobby = Lobby::new(true, to_server_sender, receiver);
                lobby.setup(game, scripts);
                lobby.run();
            });

            self.lobby_sender = Some(sender);
        } else {
            let (sender, receiver) = unbounded::<Message>();

            let to_server_sender = self.to_server_sender.clone();
            sender.send(Message::Status("Startup".to_string())).unwrap();

            let mut pool = RegionPool::new(false, to_server_sender, receiver);
            pool.add_regions(
                self.regions.values().cloned().collect(),
                self.region_behavior.clone(),
                self.behavior.clone(),
                self.systems.clone(),
                self.items.clone(),
                self.items.clone(),
                self.game.clone(),
                self.scripts.clone(),
            );
            self.pool = Some(pool);

            let (sender, receiver) = unbounded::<Message>();
            sender.send(Message::Status("Startup".to_string())).unwrap();

            let mut lobby = Lobby::new(false, self.to_server_sender.clone(), receiver);
            lobby.setup(self.game.clone(), self.scripts.clone());
            self.lobby = Some(lobby);
        }

        log::info!("Server started successfully!");

        Ok(())
    }

    /// Shutdown the system
    pub fn shutdown(&mut self) -> Result<(), String> {
        for meta in &self.metas {
            meta.sender.send(Message::Quit()).unwrap();
        }

        if let Some(lobby_sender) = &self.lobby_sender {
            lobby_sender.send(Message::Quit()).unwrap();
        }

        Ok(())
    }

    /// Only called when running on a non threaded system (web)
    pub fn tick(&mut self) -> Vec<Message> {
        let mut messages: Vec<Message> = vec![];

        if let Some(pool) = &mut self.pool {
            if let Some(msg) = pool.tick() {
                for m in msg {
                    match m {
                        Message::CharacterHasBeenTransferredInsidePool(uuid, region_id) => {
                            self.players_region_ids.insert(uuid, region_id);
                        }
                        _ => messages.push(m.clone()),
                    }
                }
            }
        }

        // Handle lobby players
        if let Some(lobby) = &mut self.lobby {
            let msg = lobby.tick();
            for m in msg {
                match m {
                    _ => messages.push(m.clone()),
                }
            }
        }

        messages
    }

    /// Gets the current messages to the server from the queue
    pub fn check_for_messages(&mut self) -> Vec<Message> {
        let mut messages: Vec<Message> = vec![];
        loop {
            if let Some(message) = self.to_server_receiver.try_recv().ok() {
                //println!("message {:?}", message);
                match message {
                    Message::CharacterHasBeenTransferredInsidePool(uuid, region_id) => {
                        self.players_region_ids.insert(uuid, region_id);
                    }
                    Message::SaveCharacter(_id, user_name, sheet) => {
                        if let Some(io) = &self.server_io {
                            let _rc = io.save_user_character(user_name.clone(), sheet);
                        }
                    }
                    _ => messages.push(message),
                }
            } else {
                break;
            }
        }
        messages
    }

    /// Create a new player character
    pub fn create_character(
        &mut self,
        id: Uuid,
        name: String,
        class: String,
        race: String,
        screen: String,
    ) {
        if let Some(position) = &self.player_default_position {
            let data = CharacterInstanceData {
                position: position.clone(),
                name: Some(name),
                tile: None,
                alignment: None,
                class: if class.is_empty() { None } else { Some(class) },
                race: if race.is_empty() { None } else { Some(race) },
                screen: Some(screen),
            };

            let mut user_name: Option<String> = None;
            if let Some(name) = self.user_names.get(&id) {
                user_name = Some(name.clone());
            }

            if self.threaded {
                self.send_message_to_region(
                    position.region,
                    Message::CreateCharacter(id, user_name, data),
                );
            } else {
                if let Some(pool) = &mut self.pool {
                    pool.create_character(id, user_name, data);
                }
            }
            self.players_region_ids.insert(id, position.region);
        }
    }

    /// Login a player character
    pub fn login_character(&mut self, id: Uuid, name: String) {
        let mut user_name: Option<String> = None;
        if let Some(name) = self.user_names.get(&id) {
            user_name = Some(name.clone());
        }

        if let Some(user_name) = user_name {
            if let Some(io) = &mut self.server_io {
                if let Some(sheet) = io.get_user_character(user_name.clone(), name.clone()).ok() {
                    if self.threaded {
                        self.send_message_to_region(
                            sheet.position.region,
                            Message::LoginCharacter(id, user_name, sheet.clone()),
                        );
                    } else {
                        if let Some(pool) = &mut self.pool {
                            pool.login_character(id, user_name, sheet.clone());
                        }
                    }
                    self.players_region_ids.insert(id, sheet.position.region);
                }
            }
        }
    }

    /// Create a new player instance
    pub fn create_player_instance(&mut self) -> Uuid {
        let uuid = uuid::Uuid::new_v4();
        if let Some(position) = &self.player_default_position {
            if self.threaded {
                self.send_message_to_region(
                    position.region,
                    Message::CreatePlayerInstance(uuid, position.clone()),
                );
            } else {
                if let Some(pool) = &mut self.pool {
                    pool.create_player_instance(uuid, position.clone());
                }
            }
            self.players_region_ids.insert(uuid, position.region);
        }
        uuid
    }

    /// Destroy a player instance
    pub fn destroy_player_instance(&mut self, uuid: Uuid) {
        if self.threaded {
            for m in &self.metas {
                _ = m.sender.send(Message::DestroyPlayerInstance(uuid));
            }
        } else {
            if let Some(pool) = &mut self.pool {
                pool.destroy_player_instance(uuid);
            }
        }

        self.remove_from_lobby(uuid);
        self.user_names.remove(&uuid);
    }

    /// Send the behavior id to debug to all pools.
    pub fn set_debug_behavior_id(&self, behavior_id: Uuid) {
        for m in &self.metas {
            m.sender
                .send(Message::SetDebugBehaviorId(behavior_id))
                .unwrap();
        }
    }

    /// Assign an action to an instance
    pub fn execute_packed_player_action(&mut self, player_uuid: Uuid, action: String) {
        if let Some(region_id) = self.players_region_ids.get(&player_uuid) {
            if let Some(action) = serde_json::from_str::<PlayerAction>(&action).ok() {
                if self.threaded {
                    let message = Message::ExecutePlayerAction(player_uuid, *region_id, action);
                    self.send_message_to_region(*region_id, message);
                } else {
                    if let Some(pool) = &mut self.pool {
                        pool.execute_player_action(player_uuid, *region_id, action);
                    }
                }
            }
        } else
        // In the Lobby ?
        {
            if let Some(action) =
                serde_json::from_str::<UserEnterGameAndCreateCharacter>(&action).ok()
            {
                // Remove from the lobby
                self.remove_from_lobby(player_uuid);

                // Enter Game
                self.create_character(
                    player_uuid,
                    action.name,
                    action.class,
                    action.race,
                    action.screen,
                )
            } else if let Some(action) =
                serde_json::from_str::<UserEnterGameWithCharacter>(&action).ok()
            {
                // Remove from the lobby
                self.remove_from_lobby(player_uuid);

                // Enter Game
                self.login_character(player_uuid, action.name)
            } else if let Some(action) = serde_json::from_str::<LoginRegisterUser>(&action).ok() {
                let mut valid_user = false;
                let mut error: Option<IOError> = None;

                if action.register {
                    if let Some(io) = &mut self.server_io {
                        let rc = io.create_user(action.user.clone(), action.password);
                        if rc.is_ok() {
                            self.user_names.insert(player_uuid, action.user.clone());
                            self.set_user_name(player_uuid, action.user.clone());
                            self.set_user_screen_name(player_uuid, action.screen);
                            self.set_user_error(player_uuid, None);
                            valid_user = true;
                        } else if let Some(err) = rc.err() {
                            error = Some(err);
                        }
                    }
                } else {
                    if let Some(io) = &mut self.server_io {
                        let rc = io.login_user(action.user.clone(), action.password);
                        if rc.is_ok() {
                            self.user_names.insert(player_uuid, action.user.clone());
                            self.set_user_name(player_uuid, action.user.clone());
                            self.set_user_screen_name(player_uuid, action.screen);
                            self.set_user_error(player_uuid, None);
                            valid_user = true;
                        } else if let Some(err) = rc.err() {
                            error = Some(err);
                        }
                    }
                }

                if let Some(error) = error {
                    let mut error_string: Option<String> = None;
                    if let Some(io) = &mut self.server_io {
                        error_string = io.error_message(error);
                    }
                    self.set_user_error(player_uuid, error_string);
                }

                if valid_user {
                    if let Some(io) = &mut self.server_io {
                        if let Some(characters) = io.list_user_characters(action.user.clone()).ok()
                        {
                            self.set_user_characters(player_uuid, characters);
                        }
                    }
                }
            } else
            // Local users for non distributed games
            if self.allow_local_users {
                if let Some(action) = serde_json::from_str::<LoginLocalUser>(&action).ok() {
                    let mut valid_user = false;
                    let mut error: Option<IOError> = None;

                    if let Some(io) = &mut self.server_io {
                        let rc = io.login_local_user(action.user.clone());
                        if rc.is_ok() {
                            self.user_names.insert(player_uuid, action.user.clone());
                            self.set_user_name(player_uuid, action.user.clone());
                            self.set_user_screen_name(player_uuid, action.screen);
                            self.set_user_error(player_uuid, None);
                            valid_user = true;
                        } else if let Some(err) = rc.err() {
                            error = Some(err);
                        }
                    }

                    if let Some(error) = error {
                        let mut error_string: Option<String> = None;
                        if let Some(io) = &mut self.server_io {
                            error_string = io.error_message(error);
                        }
                        self.set_user_error(player_uuid, error_string);
                    }

                    if valid_user {
                        if let Some(io) = &mut self.server_io {
                            if let Some(characters) =
                                io.list_user_characters(action.user.clone()).ok()
                            {
                                self.set_user_characters(player_uuid, characters);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Send a message to the given region
    pub fn send_message_to_region(&self, region_id: Uuid, message: Message) {
        for m in &self.metas {
            if m.region_ids.contains(&region_id) {
                _ = m.sender.send(message);
                break;
            }
        }
    }

    /// Creates a User struct for a user and puts him in the lobby.
    pub fn create_user(&mut self) -> Uuid {
        let user = User::new();
        let id = user.id;

        if self.threaded == false {
            if let Some(lobby) = &mut self.lobby {
                lobby.add_user(user);
            }
        } else {
            if let Some(lobby_sender) = &self.lobby_sender {
                _ = lobby_sender.send(Message::AddUserToLobby(user));
            }
        }

        id
    }

    /// Removes a user from the lobby
    pub fn remove_from_lobby(&mut self, player_uuid: Uuid) {
        if self.threaded {
            let message = Message::RemoveUserFromLobby(player_uuid);
            if let Some(lobby_sender) = &self.lobby_sender {
                lobby_sender.send(message).unwrap();
            }
        } else {
            if let Some(lobby) = &mut self.lobby {
                lobby.remove_user(player_uuid);
            }
        }
    }

    /// Sets the user name
    fn set_user_name(&mut self, player_uuid: Uuid, name: String) {
        if self.threaded {
            let message = Message::SetUserName(player_uuid, name);
            if let Some(lobby_sender) = &self.lobby_sender {
                lobby_sender.send(message).unwrap();
            }
        } else {
            if let Some(lobby) = &mut self.lobby {
                lobby.set_user_name(player_uuid, name);
            }
        }
    }

    /// Sets the users screen name
    fn set_user_screen_name(&mut self, player_uuid: Uuid, name: String) {
        if self.threaded {
            let message = Message::SetUserScreenName(player_uuid, name);
            if let Some(lobby_sender) = &self.lobby_sender {
                lobby_sender.send(message).unwrap();
            }
        } else {
            if let Some(lobby) = &mut self.lobby {
                lobby.set_user_screen_name(player_uuid, name);
            }
        }
    }

    /// Sets the users screen name
    fn set_user_error(&mut self, player_uuid: Uuid, error: Option<String>) {
        if self.threaded {
            let message = Message::SetUserError(player_uuid, error);
            if let Some(lobby_sender) = &self.lobby_sender {
                lobby_sender.send(message).unwrap();
            }
        } else {
            if let Some(lobby) = &mut self.lobby {
                lobby.set_user_error(player_uuid, error);
            }
        }
    }

    /// Sets the users characters
    fn set_user_characters(&mut self, player_uuid: Uuid, list: Vec<CharacterData>) {
        if self.threaded {
            let message = Message::SetUserCharacters(player_uuid, list);
            if let Some(lobby_sender) = &self.lobby_sender {
                lobby_sender.send(message).unwrap();
            }
        } else {
            if let Some(lobby) = &mut self.lobby {
                lobby.set_user_characters(player_uuid, list);
            }
        }
    }
}
