use lazy_static;

use std::{collections::HashMap, sync::Mutex, time::Instant};

lazy_static! {
    pub static ref CONSOLE: Console = Console {
        instants: Mutex::new(HashMap::new()),
        counters: Mutex::new(HashMap::new())
    };
}

pub struct Console {
    instants: Mutex<HashMap<String, Instant>>,
    counters: Mutex<HashMap<String, u32>>
}

impl Console {
    pub fn time(&self, name: impl Into<String>) {
        self.instants
            .lock()
            .unwrap()
            .insert(name.into(), Instant::now());
    }

    pub fn count_start(&self, name: impl Into<String>) { 
        self.counters.lock().unwrap().insert(name.into(), 0);
    }

    pub fn count(&self, name: impl Into<String>) {
        let name = name.into();
        if let Some(count) = self.counters.lock().unwrap().get_mut(&name) {
            *count += 1;
        }
    }

    pub fn count_end(&self, name: impl Into<String>) {
        let name = name.into();
        if let Some(count) = self.counters.lock().unwrap().get_mut(&name) {
            println!("count {}: {}", name, count);
        } 
    }

    pub fn time_end(&self, name: impl Into<String>) {
        if let Some((_k, _v)) = self.instants.lock().unwrap().remove_entry(&name.into()) {
            println!("{} {}micros - timer ended", _k, _v.elapsed().as_micros());
        }
    }

    pub fn log(&self, message: impl Into<String>) {
        println!("{}", message.into());
    }
}
