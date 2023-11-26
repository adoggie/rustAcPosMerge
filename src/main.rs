use std::borrow::{Borrow, BorrowMut};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::iter::{once, once_with, repeat};

use std::sync::{Arc, mpsc, Mutex};
use std::io::{Result, Error, ErrorKind};
use std::thread;

use pfollow::Runner;
// use polars::prelude::*;

mod config;
mod pfollow;
// mod database;
mod message;
mod zmqtest;
pub mod controller;
use controller::Controller;
use pfollow::SymbolTable;
// mod database;
use log::{error, warn, info, debug, trace};
use std::env;


fn main()->Result<()> {
    env_logger::init();
    info!("Starting up..");
    debug!("Current dir is {:?}", env::current_dir()?);

    let cfgfile = std::env::args().collect::<Vec<String>>().into_iter()
        .skip(1)
        .next().or(Some("./src/settings.json".to_string()));
        
    let r  = pfollow::Runner::new(&cfgfile.unwrap())?;
    let runner = Arc::new(Mutex::new(r));
    
    SymbolTable::init_from_file( &runner.lock().unwrap().get_config().symbol_file)?;
    let controller = Controller{};
    controller.run( runner.clone())?;
    Ok(())
}
