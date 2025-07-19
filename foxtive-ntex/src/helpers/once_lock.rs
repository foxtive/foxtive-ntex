use crate::FOXTIVE_NTEX;
use crate::setup::state::FoxtiveNtexState;
use foxtive::prelude::AppStateExt;
use foxtive::{FOXTIVE, FoxtiveState};
#[allow(unused_imports)]
use std::sync::{Arc, OnceLock};

pub trait FoxtiveNtexExt {
    fn app(&self) -> &FoxtiveNtexState {
        FOXTIVE_NTEX.get().unwrap()
    }

    fn foxtive(&self) -> &FoxtiveState {
        FOXTIVE.app()
    }
}

impl FoxtiveNtexExt for OnceLock<FoxtiveNtexState> {}
