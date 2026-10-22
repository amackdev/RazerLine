//! Profile management system for RazerLine.
//! Supports multiple named profiles per device with fast switching, cloning, and persistence.
//! Ported from `crates/gui/src/profiles.rs` — using local `buttons` types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::buttons::{ButtonAction, MouseButtonId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceProfileConfig {
    pub active_profile_id: String,
    pub profiles: Vec<UserProfile>,
}

impl DeviceProfileConfig {
    pub fn default_with_name(device_name: &str) -> Self {
        let default_profile = UserProfile::new_default(device_name, "Default Profile");
        Self {
            active_profile_id: default_profile.id.clone(),
            profiles: vec![default_profile],
        }
    }

    pub fn active_profile(&self) -> &UserProfile {
        self.profiles
            .iter()
            .find(|p| p.id == self.active_profile_id)
            .unwrap_or(&self.profiles[0])
    }

    #[allow(dead_code)]
    pub fn active_profile_mut(&mut self) -> &mut UserProfile {
        if let Some(idx) = self
            .profiles
            .iter()
            .position(|p| p.id == self.active_profile_id)
        {
            &mut self.profiles[idx]
        } else {
            &mut self.profiles[0]
        }
    }

    pub fn create_profile(&mut self, name: &str) -> String {
        let base = self.active_profile().clone();
        let id = format!(
            "profile_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );
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

    pub fn set_active_profile(&mut self, id: &str) -> bool {
        if self.profiles.iter().any(|p| p.id == id) {
            self.active_profile_id = id.to_string();
            true
        } else {
            false
        }
    }

    pub fn rename_profile(&mut self, id: &str, new_name: &str) -> bool {
        if let Some(idx) = self.profiles.iter().position(|p| p.id == id) {
            self.profiles[idx].name = new_name.to_string();
            true
        } else {
            false
        }
    }
}

impl Default for DeviceProfileConfig {
    fn default() -> Self {
        Self::default_with_name("Unknown Device")
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
    pub fn new_default(_device_name: &str, profile_name: &str) -> Self {
        let mut buttons = HashMap::new();
        for btn in MouseButtonId::CONFIGURABLE {
            buttons.insert(btn, btn.default_action());
        }

        Self {
            id: "default".to_string(),
            name: profile_name.to_string(),
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

    #[allow(dead_code)]
    pub fn get_button_action(&self, id: MouseButtonId) -> ButtonAction {
        if !id.is_configurable() {
            return ButtonAction::Default;
        }
        self.button_mappings
            .get(&id)
            .cloned()
            .unwrap_or_else(|| id.default_action())
    }

    #[allow(dead_code)]
    pub fn set_button_action(&mut self, id: MouseButtonId, action: ButtonAction) {
        if id.is_configurable() {
            self.button_mappings.insert(id, action);
        }
    }

    #[allow(dead_code)]
    /// Sets the number of DPI stages, clamped between 1 and 5.
    pub fn set_stage_count(&mut self, count: usize, min_dpi: u16, max_dpi: u16) {
        let count = count.clamp(1, 5);
        if count == self.dpi_stages.len() {
            return;
        }
        if count < self.dpi_stages.len() {
            self.dpi_stages.truncate(count);
            if self.current_stage_idx >= count {
                self.current_stage_idx = count.saturating_sub(1);
            }
        } else {
            while self.dpi_stages.len() < count {
                let last = self.dpi_stages.last().copied().unwrap_or(1600);
                let next = (last.saturating_add(800)).min(max_dpi).max(min_dpi);
                self.dpi_stages.push(next);
            }
        }
        self.dpi = self.dpi_stages[self.current_stage_idx];
    }

    #[allow(dead_code)]
    /// Sets the DPI value for a specific stage index.
    pub fn set_stage_dpi(&mut self, idx: usize, dpi: u16) {
        if idx < self.dpi_stages.len() {
            self.dpi_stages[idx] = dpi;
            if self.current_stage_idx == idx {
                self.dpi = dpi;
            }
        }
    }

    #[allow(dead_code)]
    /// Adds a new stage (up to 5 stages maximum).
    pub fn add_stage(&mut self, min_dpi: u16, max_dpi: u16) -> bool {
        if self.dpi_stages.len() >= 5 {
            return false;
        }
        let last = self.dpi_stages.last().copied().unwrap_or(1600);
        let next = (last.saturating_add(800)).min(max_dpi).max(min_dpi);
        self.dpi_stages.push(next);
        true
    }

    #[allow(dead_code)]
    /// Removes a stage by index (keeping at least 1 stage).
    pub fn remove_stage(&mut self, idx: usize) -> bool {
        if self.dpi_stages.len() <= 1 || idx >= self.dpi_stages.len() {
            return false;
        }
        self.dpi_stages.remove(idx);
        if self.current_stage_idx >= self.dpi_stages.len() {
            self.current_stage_idx = self.dpi_stages.len().saturating_sub(1);
        }
        self.dpi = self.dpi_stages[self.current_stage_idx];
        true
    }

    #[allow(dead_code)]
    /// Sorts stages in ascending order while keeping the current DPI active.
    pub fn sort_stages(&mut self) {
        let current_dpi = self.dpi;
        self.dpi_stages.sort_unstable();
        if let Some(pos) = self.dpi_stages.iter().position(|&s| s == current_dpi) {
            self.current_stage_idx = pos;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_profile_stages() {
        let p = UserProfile::new_default("Test", "Default");
        assert_eq!(p.dpi_stages, vec![400, 800, 1600, 3200, 6400]);
        assert_eq!(p.current_stage_idx, 2);
        assert_eq!(p.dpi, 1600);
    }

    #[test]
    fn test_stage_count_adjustment() {
        let mut p = UserProfile::new_default("Test", "Default");
        p.set_stage_count(3, 100, 20000);
        assert_eq!(p.dpi_stages.len(), 3);
        assert_eq!(p.dpi_stages, vec![400, 800, 1600]);
        assert_eq!(p.current_stage_idx, 2);
        assert_eq!(p.dpi, 1600);

        p.set_stage_count(2, 100, 20000);
        assert_eq!(p.dpi_stages.len(), 2);
        assert_eq!(p.current_stage_idx, 1);
        assert_eq!(p.dpi, 800);

        p.set_stage_count(4, 100, 20000);
        assert_eq!(p.dpi_stages.len(), 4);
    }

    #[test]
    fn test_stage_dpi_edits() {
        let mut p = UserProfile::new_default("Test", "Default");
        p.set_stage_dpi(2, 1800);
        assert_eq!(p.dpi_stages[2], 1800);
        assert_eq!(p.dpi, 1800);

        p.set_stage_dpi(0, 500);
        assert_eq!(p.dpi_stages[0], 500);
        assert_eq!(p.dpi, 1800); // active was stage 2, untouched
    }

    #[test]
    fn test_add_and_remove_stages() {
        let mut p = UserProfile::new_default("Test", "Default");
        assert!(!p.add_stage(100, 20000)); // already at max 5

        assert!(p.remove_stage(0)); // remove 400
        assert_eq!(p.dpi_stages.len(), 4);
        assert_eq!(p.dpi_stages, vec![800, 1600, 3200, 6400]);

        assert!(p.add_stage(100, 20000));
        assert_eq!(p.dpi_stages.len(), 5);

        // Remove down to 1
        p.remove_stage(0);
        p.remove_stage(0);
        p.remove_stage(0);
        p.remove_stage(0);
        assert_eq!(p.dpi_stages.len(), 1);
        assert!(!p.remove_stage(0)); // cannot remove last remaining stage
    }

    #[test]
    fn test_sort_stages() {
        let mut p = UserProfile {
            dpi_stages: vec![3200, 400, 1600],
            current_stage_idx: 0,
            dpi: 3200,
            ..UserProfile::new_default("Test", "Default")
        };

        p.sort_stages();
        assert_eq!(p.dpi_stages, vec![400, 1600, 3200]);
        assert_eq!(p.current_stage_idx, 2);
        assert_eq!(p.dpi, 3200);
    }
}
