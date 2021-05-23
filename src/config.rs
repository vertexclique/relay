use serde::*;
use lever::prelude::LOTable;
use crate::error::*;
use std::fs::File;
use std::io::prelude::*;
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct RouterConfig {
    hot_update_port: usize,
    routing_table: HashMap<String, String>,
}

pub struct Config {
    config_file: String,
    hot_update_port: usize,
    routing_table: LOTable<String, String>
}

impl Default for Config {
    fn default() -> Self {
        Self {
            config_file: String::from("relay.json"),
            hot_update_port: 32023,
            routing_table: LOTable::default()
        }
    }
}

impl Config {
    pub fn with_file<T>(mut self, path: T) -> Self
    where
        T: AsRef<str>
    {
        self.config_file = path.as_ref().to_string();
        self
    }

    pub fn with_hot_update_port(mut self, hot_update_port: usize) -> Self {
        self.hot_update_port = hot_update_port;
        self
    }

    pub fn with_routing_table(mut self, routing_table: LOTable<String, String>) -> Self
    {
        self.routing_table = routing_table;
        self
    }

    pub fn build(mut self) -> Result<Self> {
        let mut file = File::open(self.config_file.as_str())?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let rconf: RouterConfig = serde_json::from_str(contents.as_mut_str())?;

        self.hot_update_port = rconf.hot_update_port;
        rconf.routing_table.iter().for_each(|(k, v)| {
            let _ = self.routing_table.insert(k.to_owned(), v.to_owned());
        });

        Ok(self)
    }
}
