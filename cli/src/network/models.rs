use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Network driver types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum NetworkDriver {
    Bridge,
    Host,
    None,
    #[serde(untagged)]
    Custom(String),
}

impl NetworkDriver {
    pub fn as_str(&self) -> &str {
        match self {
            NetworkDriver::Bridge => "bridge",
            NetworkDriver::Host => "host",
            NetworkDriver::None => "none",
            NetworkDriver::Custom(s) => s.as_str(),
        }
    }
}

impl Default for NetworkDriver {
    fn default() -> Self {
        NetworkDriver::Bridge
    }
}

/// Network information
#[derive(Debug, Clone, Serialize)]
pub struct Network {
    pub id: String,
    pub name: String,
    pub driver: String,
    pub subnet: Option<String>,
    pub gateway: Option<String>,
    pub internal: bool,
    pub labels: HashMap<String, String>,
}

/// Network creation/update configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub name: String,

    #[serde(default)]
    pub driver: NetworkDriver,

    pub subnet: Option<String>,
    pub gateway: Option<String>,

    #[serde(default)]
    pub internal: bool,

    #[serde(default)]
    pub labels: HashMap<String, String>,
}

impl NetworkConfig {
    pub fn new(name: String) -> Self {
        Self {
            name,
            driver: NetworkDriver::Bridge,
            subnet: None,
            gateway: None,
            internal: false,
            labels: HashMap::new(),
        }
    }

    pub fn with_driver(mut self, driver: NetworkDriver) -> Self {
        self.driver = driver;
        self
    }

    pub fn with_subnet(mut self, subnet: String) -> Self {
        self.subnet = Some(subnet);
        self
    }

    pub fn with_gateway(mut self, gateway: String) -> Self {
        self.gateway = Some(gateway);
        self
    }

    pub fn with_internal(mut self, internal: bool) -> Self {
        self.internal = internal;
        self
    }
}
