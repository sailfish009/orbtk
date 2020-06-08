use std::{any::Any, collections::HashMap};

use ron::{de::from_str, Value};
use serde_derive::{Deserialize, Serialize};

pub struct Selector {
    pub style: Option<String>,
    pub state: Option<String>,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Theme {
    name: String,
    styles: HashMap<String, Style>,
}

impl<'a> Theme {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn property(&'a self, property: &str, selector: &Selector) -> Option<&'a Value> {
        if let Some(style) = &selector.style {
            if let Some(style) = self.styles.get(style) {
                return self.get_property(property, style, selector);
            }
        }

        if let Some(base_style) = self.styles.get("base") {
            return self.get_property(property, base_style, selector);
        }

        None
    }

    // todo check based on style!!!!

    fn get_property(
        &'a self,
        property: &str,
        style: &'a Style,
        selector: &Selector,
    ) -> Option<&'a Value> {
        if let Some(state) = &selector.state {
            if let Some(properties) = style.states.get(state) {
                return self.get_property_value(property, properties);
            }

            if let Some(base_style) = self.styles.get("base") {
                if let Some(properties) = base_style.states.get(state) {
                    return self.get_property_value(property, properties);
                }
            }
        }

        let value = self.get_property_value(property, &style.properties);

        if value.is_some() {
            return value;
        }

        if let Some(base_style) = self.styles.get("base") {
            return self.get_property_value(property, &base_style.properties);
        }

        None
    }

    fn get_property_value(
        &self,
        property: &str,
        properties: &'a HashMap<String, Value>,
    ) -> Option<&'a Value> {
        properties.get(property)
    }
}

impl From<&str> for Theme {
    fn from(s: &str) -> Self {
        from_str(s).unwrap()
    }
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Style {
    // set default string to base style
    #[serde(default)]
    base: String,
    #[serde(default)]
    properties: HashMap<String, Value>,
    #[serde(default)]
    states: HashMap<String, HashMap<String, Value>>,
}
