use std::cell::RefCell;
use std::sync::OnceLock;
use std::time::Instant;

use ic_stable_structures::memory_manager::MemoryId;
use ic_stable_structures::memory_manager::MemoryManager;
use ic_stable_structures::memory_manager::VirtualMemory;
use ic_stable_structures::BTreeMap;
use ic_stable_structures::DefaultMemoryImpl;

use crate::app::App;
use crate::app::AppID;
use crate::app::AppUsage;
use crate::developer::Developer;
use crate::developer::DeveloperID;
use crate::error::Error;
use crate::settings::Settings;
use crate::Result;

// A new memory should be created for every additional stable structure.
const USERS_BTREE: MemoryId = MemoryId::new(0);
const APPS_BTREE: MemoryId = MemoryId::new(1);

pub type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    // The memory manager is used for simulating multiple memories. Given a `MemoryId` it can
    // return a memory that can be used by stable structures.
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
}

fn get_users_btree_memory() -> Memory {
    MEMORY_MANAGER.with(|m| m.borrow().get(USERS_BTREE))
}

fn get_apps_btree_memory() -> Memory {
    MEMORY_MANAGER.with(|m| m.borrow().get(APPS_BTREE))
}

pub struct State {
    settings: OnceLock<Settings>,
    developers: BTreeMap<DeveloperID, Developer, Memory>,

    // TODO: Refactor when there is support for nested structure in `ic_stable_structures`.
    // See: https://github.com/dfinity/stable-structures/issues/215#issuecomment-2090315537
    apps: BTreeMap<AppID, App, Memory>,
    pub icp_cycles_exchange_rate: Option<(u64, Instant)>,
}

impl State {
    pub fn init_settings(&mut self, settings: Settings) {
        ic_cdk::println!("State.settings initialized");
        ic_cdk::println!("ThreadID: {:?}", std::thread::current().id());
        self.settings.get_or_init(|| settings);
    }

    pub fn settings(&self) -> &Settings {
        ic_cdk::println!("State.settings:{:?}", self.settings);
        ic_cdk::println!("ThreadID: {:?}", std::thread::current().id());
        self.settings
            .get()
            .expect("Canister is not initialized correctly")
    }

    pub fn register_developer(
        &mut self,
        developer_id: DeveloperID,
        developer: Developer,
    ) -> Result<()> {
        self.developers.insert(developer_id, developer);
        Ok(())
    }

    pub fn get_developer(&self, developer_id: &DeveloperID) -> Result<Developer> {
        self.developers
            .get(developer_id)
            .ok_or(Error::DeveloperAccountNotFound)
    }

    pub fn get_app_of_developer(
        &self,
        developer_id: &DeveloperID,
        app_id: &AppID,
    ) -> Result<Option<App>> {
        let developer = self.get_developer(developer_id)?;
        Ok(developer
            .apps
            .contains(app_id)
            .then(|| self.apps.get(app_id))
            .flatten())
    }

    pub fn get_apps_of_developer(&self, developer_id: &DeveloperID) -> Result<Vec<(AppID, App)>> {
        let developer = self.get_developer(developer_id)?;
        Ok(developer
            .apps
            .into_iter()
            .filter_map(|app_id| self.apps.get(&app_id).map(|a| (app_id, a)))
            .collect())
    }

    pub fn register_app(&mut self, app_id: AppID, app: App) {
        let developer_id = app.developer_id;
        let mut developer = self
            .developers
            .get(&developer_id)
            .expect("Developer not found");

        self.apps.insert(app_id, app);

        developer.apps.push(app_id);
        self.developers.insert(developer_id, developer);
    }

    pub fn remove_app(&mut self, app_id: AppID) -> Result<()> {
        if let Some(app) = self.apps.remove(&app_id) {
            let mut developer = self
                .developers
                .get(&app.developer_id)
                .expect("Developer not found");

            developer.apps.retain(|a| *a != app_id);
            self.developers.insert(app.developer_id, developer);
        }
        Ok(())
    }

    pub fn get_app(&self, app_id: &AppID) -> Result<App> {
        self.apps.get(app_id).ok_or(Error::AppNotFound)
    }

    pub fn register_usage(&mut self, app_id: AppID, usage: AppUsage) -> Result<()> {
        let mut app = self.apps.get(&app_id).ok_or(Error::AppNotFound)?;
        app.usages.push(usage);
        self.apps.insert(app_id, app);
        Ok(())
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            settings: OnceLock::new(),
            developers: BTreeMap::init(get_users_btree_memory()),
            apps: BTreeMap::init(get_apps_btree_memory()),
            icp_cycles_exchange_rate: None,
        }
    }
}

thread_local! {
    pub static STATE: RefCell<State> = RefCell::new(State::default());
}
