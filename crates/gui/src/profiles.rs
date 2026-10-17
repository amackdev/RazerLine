//! Profile management system for RazerLine.
//! Supports multiple named profiles per device with fast switching, cloning, and persistence.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::buttons::{ButtonAction, MouseButtonId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceProfileConfig {
    pub active_profile_id: String,
    pub profiles: Vec<UserProfile>,
}

impl Default for DeviceProfileConfig {
    fn default() -> Self {
        let default_profile = UserProfile::new_default("Default Profile");
        Self {
            active_profile_id: default_profile.id.clone(),
            profiles: vec![default_profile],
        }
    }
}

impl DeviceProfileConfig {
    pub fn active_profile(&self) -> &UserProfile {
        self.profiles
            .iter()
            .find(|p| p.id == self.active_profile_id)
            .unwrap_or(&self.profiles[0])
    }

    pub fn active_profile_mut(&mut self) -> &mut UserProfile {
        if let Some(idx) = self.profiles.iter().position(|p| p.id == self.active_profile_id) {
            &mut self.profiles[idx]
        } else {
            &mut self.profiles[0]
        }
    }

    pub fn create_profile(&mut self, name: &str) -> String {
        let base = self.active_profile().clone();
        let id = format!("profile_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());
        let mut new_profile = base;
        new_profile.id = id.clone();
        new_profile.name = name.to_string();
        self.profiles.push(new_profile);
        self.active_profile_id = id.clone();
        id
    }

    pub fn duplicate_active(&mut self) -> String {
        let current_name = self.active_profile().name.clone();
        let new_name = format!("{current_name} (Copy)");
        self.create_profile(&new_name)
    }

    pub fn delete_profile(&mut self, id: &str) -> bool {
        if self.profiles.len() <= 1 {
            return false; // Always keep at least one profile
        }
        if let Some(pos) = self.profiles.iter().position(|p| p.id == id) {
            self.profiles.remove(pos);
            if self.active_profile_id == id {
                self.active_profile_id = self.profiles[0].id.clone();
            }
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: String,
    pub name: String,
    pub dpi: u16,
    pub dpi_stages: Vec<u16>,
    pub current_stage_idx: usize,
    pub poll_rate_hz: Option<u32>,
    pub brightness: u8,
    pub led_color: [u8; 3],
    pub sync_zones: bool,
    pub effect: String,
    pub matrix_color: Option<[u8; 3]>,
    pub button_mappings: HashMap<MouseButtonId, ButtonAction>,
}

impl UserProfile {
    pub fn new_default(name: &str) -> Self {
        let mut buttons = HashMap::new();
        for btn in MouseButtonId::CONFIGURABLE {
            buttons.insert(btn, btn.default_action());
        }

        Self {
            id: "default".to_string(),
            name: name.to_string(),
            dpi: 1600,
            dpi_stages: vec![400, 800, 1600, 3200, 6400],
            current_stage_idx: 2,
            poll_rate_hz: Some(1000),
            brightness: 100,
            led_color: [64, 209, 128],
            sync_zones: true,
            effect: "static".to_string(),
            matrix_color: Some([64, 209, 128]),
            button_mappings: buttons,
        }
    }

    pub fn get_button_action(&self, id: MouseButtonId) -> ButtonAction {
        if !id.is_configurable() {
            return ButtonAction::Default;
        }
        self.button_mappings
            .get(&id)
            .cloned()
            .unwrap_or_else(|| id.default_action())
    }

    pub fn set_button_action(&mut self, id: MouseButtonId, action: ButtonAction) {
        if id.is_configurable() {
            self.button_mappings.insert(id, action);
        }
    }
}
