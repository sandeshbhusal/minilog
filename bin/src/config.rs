#[derive(Debug, Serialize, Deserialize)]
pub struct ModuleConfiguration {
    pub uses: String,
    #[serde(flatten)]
    pub configuration: HashMap<String, ConfigValue>,
}

impl ModuleConfiguration {
    pub(crate) fn get_mandatory(&self, key: &str) -> anyhow::Result<ConfigValue> {
        self.get_optional(key)
            .ok_or_else(|| anyhow::anyhow!("Mandatory configuration keyword `{}` missing.", key))
    }

    fn get_optional(&self, key: &str) -> Option<ConfigValue> {
        self.configuration.get(key).map(|t| t.clone())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfigValue {
    Usize(usize),
    String(String),
    Float(f64),
    Integer(i32),
}

use std::{collections::HashMap, path::Path};

use serde::{Deserialize, Serialize};

impl TryFrom<ConfigValue> for String {
    type Error = anyhow::Error;

    fn try_from(value: ConfigValue) -> anyhow::Result<std::string::String> {
        if let ConfigValue::String(str) = value {
            Ok(str)
        } else {
            anyhow::bail!("Type {:?} cannot be converted to string.", value)
        }
    }
}

impl TryFrom<ConfigValue> for usize {
    type Error = anyhow::Error;

    fn try_from(value: ConfigValue) -> anyhow::Result<usize> {
        if let ConfigValue::Usize(us) = value {
            Ok(us)
        } else {
            anyhow::bail!("Type {:?} cannot be converted to usize.", value)
        }
    }
}
impl TryFrom<ConfigValue> for f64 {
    type Error = anyhow::Error;

    fn try_from(value: ConfigValue) -> anyhow::Result<f64> {
        if let ConfigValue::Float(float) = value {
            Ok(float)
        } else {
            anyhow::bail!("Type {:?} cannot be converted to float.", value)
        }
    }
}

impl TryFrom<ConfigValue> for i32 {
    type Error = anyhow::Error;

    fn try_from(value: ConfigValue) -> anyhow::Result<i32> {
        if let ConfigValue::Integer(int) = value {
            Ok(int)
        } else {
            anyhow::bail!("Type {:?} cannot be converted to integer.", value)
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Configuration {
    pub modules: HashMap<String, ModuleConfiguration>,
    pub routes: HashMap<String, Route>,
}

pub fn read_configuration(mut path: impl AsRef<Path>) -> anyhow::Result<Configuration> {
    let buffer = std::fs::read_to_string(&mut path)?;
    let config = toml::from_str::<Configuration>(&buffer)?;
    Ok(config)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Route {
    pub from: Vec<String>,
    pub to: Vec<String>,
}
