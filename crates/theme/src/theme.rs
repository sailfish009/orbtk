use std::{any::Any, collections::HashMap};

use ron::{de::from_str, Value};
use serde_derive::{Deserialize, Serialize};

static BASE_STYLE: &str = "base";
static RESOURCE_KEY: &str = "$";

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
            dirty: true,
        }
    }

    pub fn set_state(&mut self, state: impl Into<String>) {
        self.state = Some(state.into());
    }
}

/// Used to store and read properties that could be requested by a given property name and a selector.
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Theme {
    #[serde(default)]
    styles: HashMap<String, Style>,
    #[serde(default)]
    resources: HashMap<String, Value>,
}

impl<'a> Theme {
    /// Extends the given theme with a other theme. Replaces the current name with name of other.
    /// If a style with the same key is on other, it will replace the style in the current theme.
    pub fn extend(mut self, other: Theme) -> Self {
        let mut other = other;

        for style in other.styles.drain() {
            self.styles.insert(style.0, style.1);
        }

        for resource in other.resources.drain() {
            self.resources.insert(resource.0, resource.1);
        }

        self
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
        let property = properties.get(property)?;

        // load from resources if the value is a key.
        if let Ok(key) = property.clone().into_rust::<String>() {
            if key.starts_with(RESOURCE_KEY) {
                return Some(self.resources.get(&key.replace(RESOURCE_KEY, ""))?.clone());
            }
        }

        Some(property.clone())
    }

    pub fn properties(&'a self, selector: &'a Selector) -> PropertyIterator {
        PropertyIterator {
            theme: self,
            selector,
        }
    }
}

impl From<&str> for Theme {
    fn from(s: &str) -> Self {
        from_str(s).unwrap()
    }
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

pub struct PropertyIterator<'a> {
    theme: &'a Theme,
    selector: &'a Selector,
}

impl<'a> Iterator for PropertyIterator<'a> {
    type Item = (String, Value);

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

pub struct ThemeX {
    styles: HashMap<String, StyleX>,
}

impl ThemeX {
    pub fn from_config(theme: Theme) -> Self {
        let mut styles = HashMap::new();

        for style_key in theme.styles.keys() {
            let mut properties = HashMap::new();
            ThemeX::read_properties(style_key, &theme, &mut properties);

            let mut states = HashMap::new();

            for state_key in theme.styles.get(style_key).unwrap().states.keys() {
                let mut state = HashMap::new();
                ThemeX::read_states(style_key,state_key, &theme, &mut state);
                states.insert(state_key.clone(), state);
            }     

            styles.insert(style_key.clone(), StyleX { properties, states});
        }

        ThemeX { styles }
    }

    pub fn style(&self, key: &str) -> Option<&StyleX> {
        self.styles.get(key)
    }

    pub fn properties<'a>(&'a self, selector: &Selector) -> Option<&'a HashMap<String, Value>> {
        if !selector.dirty {
            return None;
        }

        if let Some(style) = &selector.style {
            if let Some(state) = &selector.state {
                return self.styles.get(style)?.states.get(state)
            }

            return Some(&self.styles.get(style)?.properties);
        }

        return None
    }

    fn read_properties(key: &String, theme: &Theme, properties: &mut HashMap<String, Value>) {
        if key.is_empty() {
            return;
        }

        if let Some(style) = theme.styles.get(key) {
            ThemeX::read_properties(&style.base, theme, properties);

            for (key, value) in &style.properties {
                ThemeX::read_property(key, value, theme, properties);
            }
        }
    }

    fn read_states(
        style_key: &String,
        state_key: &String,
        theme: &Theme,
        states: &mut HashMap<String, Value>,
    ) {
        if style_key.is_empty() || state_key.is_empty() {
            return;
        }

        if let Some(style) = theme.styles.get(style_key) {
            ThemeX::read_states(&style.base,  state_key, theme, states);

            for (key, value) in &style.properties {
                ThemeX::read_property(key, value, theme, states);
            }

            if let Some(state) = style.states.get(state_key) {
                for (key, value) in state {
                    ThemeX::read_property(key, value, theme, states);
                }
            }
        }
    }

    fn read_property(key: &String, value: &Value, theme: &Theme, map: &mut HashMap<String, Value>) {
        if let Ok(value) = value.clone().into_rust::<String>() {
            if value.starts_with(RESOURCE_KEY) {
                if let Some(value) = theme.resources.get(&value.replace(RESOURCE_KEY, "")) {
                    map.insert(key.clone(), value.clone());
                }
            } else {
                map.insert(key.clone(), Value::String(value));
            }
        } else {
            map.insert(key.clone(), value.clone());
        }
    }
}

pub struct StyleX {
    properties: HashMap<String, Value>,
    states: HashMap<String, HashMap<String, Value>>,
}
