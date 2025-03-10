#[allow(unused_imports)]
use std::sync::{Arc, OnceLock};
use foxtive::{FoxtiveState, FOXTIVE};
use foxtive::prelude::OnceLockHelper;
use crate::setup::state::FoxtiveNtexState;
use crate::FOXTIVE_WEB;

pub trait WebOnceLockHelper {
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

impl WebOnceLockHelper for OnceLock<FoxtiveNtexState> {}
