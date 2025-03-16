use crate::setup::state::FoxtiveNtexState;
use crate::FOXTIVE_WEB;
use foxtive::prelude::OnceLockHelper;
use foxtive::{FoxtiveState, FOXTIVE};
#[allow(unused_imports)]
use std::sync::{Arc, OnceLock};

pub trait FoxtiveNtexExt {
    fn app(&self) -> &FoxtiveNtexState {
        FOXTIVE_WEB.get().unwrap()
    }

    fn foxtive(&self) -> &FoxtiveState {
        FOXTIVE.app()
    }

    fn front_url(&self, url: &str) -> String {
        self.foxtive().frontend(url)
    }
}

impl FoxtiveNtexExt for OnceLock<FoxtiveNtexState> {}
