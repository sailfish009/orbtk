use std::{any::Any, collections::HashMap};

use ron::{de::from_str, Value};
use serde_derive::{Deserialize, Serialize};

pub static BASE_STYLE: &str = "base";

/// The selector is used to read a property value from the `Theme`.
#[derive(Debug, Clone, Default)]
pub struct Selector {
    /// Represents the key of a style.
    pub style: Option<String>,

    /// Used to reference the state property list of the given style.
    pub state: Option<String>,

    pub dirty: bool,
}

impl Selector {
    pub fn new(style: impl Into<String>) -> Self {
        Selector {
            style: Some(style.into()),
            state: None,
            dirty: true
        }
    }

    pub fn set_state(&mut self, state: impl Into<String>) {
        self.state = Some(state.into());
    }
}

/// Used to store and read properties that could be requested by a given property name and a selector.
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Theme {
    name: String,
    styles: HashMap<String, Style>,
}

impl<'a> Theme {
    /// Gets the name of the theme.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Extends the given theme with a other theme. Replaces the current name with name of other.
    /// If a style with the same key is on other, it will replace the style in the current theme. 
    pub fn extend(&mut self, other: Theme) {
        let mut other = other;
        self.name = other.name;

        for style in other.styles.drain() {
            self.styles.insert(style.0, style.1);
        }
    }

    /// Gets a property by the given name and a selector.
    pub fn property(&'a self, property: &str, selector: &Selector) -> Option<Value> {
        if let Some(style) = &selector.style {
            if let Some(style) = self.styles.get(style) {
                return self.get_property(property, style, selector);
            }
        }

        // if there is no style read value from base style.
        if let Some(base_style) = self.styles.get(BASE_STYLE) {
            return self.get_property(property, base_style, selector);
        }

        None
    }

    fn get_property(
        &'a self,
        property: &str,
        style: &'a Style,
        selector: &Selector,
    ) -> Option<Value> {
        // state properties has the most priority
        if let Some(state) = &selector.state {
            if let Some(properties) = style.states.get(state) {
                return self.get_property_value(property, properties);
            }
 
            // load state properties from based style if there are no other states (recursive through base style).
            if style.base.is_empty() {
                return None;
            }

            if let Some(base_style) = self.styles.get(&style.base) {
                if let Some(properties) = base_style.states.get(state) {
                    return self.get_property_value(property, properties);
                }
            }
        }

        let value = self.get_property_value(property, &style.properties);

        if value.is_some() {
            return value;
        }

        // load properties from based style if there are no other states (recursive through base style).
        if style.base.is_empty() {
            return None;
        }

        if let Some(base_style) = self.styles.get(&style.base) {
            return self.get_property(property, base_style, selector);
        }

        None
    }

    fn get_property_value(
        &self,
        property: &str,
        properties: &'a HashMap<String, Value>,
    ) -> Option<Value> {
        Some(properties.get(property)?.clone())
    }
}

impl From<&str> for Theme {
    fn from(s: &str) -> Self {
        from_str(s).unwrap()
    }
}

fn default_style() -> String {
    BASE_STYLE.to_string()
}

/// Defines a style. A style could be base on other styles and contains a list for properties
/// and a list of state properties.
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Style {
    // set default string to base style
    #[serde(default)]
    base: String,
    #[serde(default)]
    states: HashMap<String, HashMap<String, Value>>,
    #[serde(default)]
    properties: HashMap<String, Value>,
}
