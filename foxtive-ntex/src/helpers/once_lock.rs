use crate::setup::state::FoxtiveNtexState;
use crate::FOXTIVE_NTEX;
use foxtive::prelude::OnceLockHelper;
use foxtive::{FoxtiveState, FOXTIVE};
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
