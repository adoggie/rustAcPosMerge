use std::sync::{Arc,Mutex};
use std::thread;
use crate::pfollow::*;

pub struct Controller{
    // name :String ,
}

// write a function macd()  to calculate macd 

impl Controller{
    pub fn run(&self,runner:Arc<Mutex<Runner>>)->std::io::Result<()>{
        runner.lock().unwrap().run()?;     
        Ok(())
    }
    
}
