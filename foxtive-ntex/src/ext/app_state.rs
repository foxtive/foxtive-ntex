use crate::FOXTIVE_NTEX;
use crate::setup::state::FoxtiveNtexState;
use foxtive::prelude::AppStateExt;
use foxtive::{FOXTIVE, FoxtiveState};
use std::sync::OnceLock;

/// Get a reference to custom state by type.
pub fn fox_state<T: Send + Sync + 'static>() -> Result<&'static T, String> {
    let state = FOXTIVE_NTEX.get().ok_or("FoxtiveNtexState not initialized")?;
    
    state.iter_custom_state()
        .find_map(|(_key, value)| value.downcast_ref::<T>())
        .ok_or_else(|| format!("Custom state of type {} not found", std::any::type_name::<T>()))
}

pub trait FoxtiveNtexExt {
    fn app(&self) -> &FoxtiveNtexState {
        FOXTIVE_NTEX.get().unwrap()
    }

    fn foxtive(&self) -> &FoxtiveState {
        FOXTIVE.app()
    }
}

impl FoxtiveNtexExt for OnceLock<FoxtiveNtexState> {}
