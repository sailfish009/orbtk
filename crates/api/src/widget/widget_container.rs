use std::any::TypeId;

use ron::Value;

use crate::{
    css_engine::*,
    prelude::*,
    utils::{Brush, String16, Thickness},
};

use dces::prelude::{Component, Entity, EntityComponentManager};

/// The `WidgetContainer` wraps the entity of a widget and provides access to its properties, its children properties and its parent properties.
pub struct WidgetContainer<'a> {
    ecm: &'a mut EntityComponentManager<Tree, StringComponentStore>,
    current_node: Entity,
    theme: &'a ThemeValue,
    _theme: &'a crate::theme::Theme,
}

impl<'a> WidgetContainer<'a> {
    /// Creates a new widget container for the given `entity`.
    pub fn new(
        root: Entity,
        ecm: &'a mut EntityComponentManager<Tree, StringComponentStore>,
        theme: &'a ThemeValue,
        _theme: &'a crate::theme::Theme,
    ) -> Self {
        WidgetContainer {
            ecm,
            current_node: root,
            theme,
            _theme,
        }
    }

    /// Gets the entity of the widget.
    pub fn entity(&self) -> Entity {
        self.current_node
    }

    /// Gets the property.
    ///
    /// # Panics
    ///
    /// Panics if the widget does not contains the property.
    pub fn get<P>(&self, key: &str) -> &P
    where
        P: Clone + Component,
    {
        if let Ok(property) = self.ecm.component_store().get::<P>(key, self.current_node) {
            return property;
        }

        panic!(
            "Entity {} does not contain property type {:?} with key: {}",
            self.current_node.0,
            TypeId::of::<P>(),
            key
        );
    }

    /// Gets a mutable reference of the property of type `P`.
    ///
    /// # Panics
    ///
    /// Panics if the widget does not contains the property.
    pub fn get_mut<P>(&mut self, key: &str) -> &mut P
    where
        P: Clone + Component,
    {
        if let Ok(property) = self
            .ecm
            .component_store_mut()
            .get_mut::<P>(key, self.current_node)
        {
            return property;
        }

        panic!(
            "Entity {} does not contain property type {:?}, with key: {}",
            self.current_node.0,
            TypeId::of::<P>(),
            key
        );
    }

    /// Clones the property. If the property does not exists for the widget the
    /// default of the property value will be returned,
    pub fn clone_or_default<P>(&self, key: &str) -> P
    where
        P: Clone + Component + Default,
    {
        if let Ok(property) = self.ecm.component_store().get::<P>(key, self.current_node) {
            return property.clone();
        }

        P::default()
    }

    /// Clones the property.
    ///
    /// # Panics
    ///
    /// Panics if the widget does not contains the property.
    pub fn clone<P>(&self, key: &str) -> P
    where
        P: Clone + Component,
    {
        if let Ok(property) = self.ecm.component_store().get::<P>(key, self.current_node) {
            return property.clone();
        }

        panic!(
            "Entity {} does not contain property type {:?}, with key: {}",
            self.current_node.0,
            TypeId::of::<P>(),
            key
        );
    }

    /// Clones the property of type `P` from the given widget entity. If the entity does
    /// not exists or it doesn't have a component of type `P` `None` will be returned.
    pub fn try_clone<P>(&self, key: &str) -> Option<P>
    where
        P: Clone + Component,
    {
        if let Ok(property) = self.ecm.component_store().get::<P>(key, self.current_node) {
            return Some(property.clone());
        }

        None
    }

    /// Sets the property of type `P`.
    ///
    /// # Panics
    ///
    /// Panics if the widget does not contains the property.
    pub fn set<P>(&mut self, key: &str, value: P)
    where
        P: Component + Clone,
    {
        if let Ok(property) = self
            .ecm
            .component_store_mut()
            .get_mut::<P>(key, self.current_node)
        {
            *property = value;
            return;
        }

        panic!(
            "Entity {} does not contain property type {:?}",
            self.current_node.0,
            TypeId::of::<P>()
        );
    }

    /// Returns `true` if the widget has a property of type `P` otherwise `false`.
    pub fn has<P>(&self, key: &str) -> bool
    where
        P: Clone + Component,
    {
        self.ecm
            .component_store()
            .get::<P>(key, self.current_node)
            .is_ok()
    }

    /// Returns a reference of a property of type `P` from the given widget entity. If the entity does
    /// not exists or it doesn't have a component of type `P` `None` will be returned.
    pub fn try_get<P: Component>(&self, key: &str) -> Option<&P> {
        self.ecm
            .component_store()
            .get::<P>(key, self.current_node)
            .ok()
    }

    /// Returns a mutable reference of a property of type `P` from the given widget entity. If the entity does
    /// not exists or it doesn't have a component of type `P` `None` will be returned.
    pub fn try_get_mut<P: Component>(&mut self, key: &str) -> Option<&mut P> {
        self.ecm
            .component_store_mut()
            .get_mut::<P>(key, self.current_node)
            .ok()
    }

    /// Checks if the given value is equal to the given property.
    pub fn eq<P: Component + PartialEq>(&self, key: &str, other: &P) -> bool {
        if let Some(value) = self.try_get::<P>(key) {
            return value.eq(other);
        }

        false
    }

    fn update_internal_theme_by_state(&mut self, force: bool, entity: &Entity) {
        for child in &(self.ecm.entity_store().children.clone())[&entity] {
            self.update_internal_theme_by_state(force, child);
        }

        self.current_node = *entity;
        

        // crate::shell::CONSOLE.time("state update");
        if let Some(mut selector) = self.try_clone::<crate::theme::Selector>("_selector") {
            if let Some(focus) = self.try_get::<bool>("focused") {
                update_state("focused", *focus, &mut selector);
            }

            if let Some(selected) = self.try_get::<bool>("selected") {
                update_state("selected", *selected, &mut selector);
            }

            if let Some(pressed) = self.try_get::<bool>("pressed") {
                if *pressed {
                    crate::shell::CONSOLE.count_start("updates");
                }
                update_state("pressed", *pressed, &mut selector);
            }

            if let Some(enabled) = self.try_get::<bool>("enabled") {
                update_state("disabled", !enabled, &mut selector);
            }

            if let Some(text) = self.try_get::<String16>("text") {
                update_state("empty", text.is_empty(), &mut selector);
            }

            if let Some(expanded) = self.try_get::<bool>("expanded") {
                update_state("expanded", !expanded, &mut selector);
            }
            // crate::shell::CONSOLE.time_end("state update");

            // crate::shell::CONSOLE.time("reload properties");

            self.set("_selector", selector);
            if self.get::<crate::theme::Selector>("_selector").dirty || force {
                self.update_properties_by_theme();
            }
            // crate::shell::CONSOLE.time_end("reload properties");
        }
    }

    /// Updates the theme by the inner state e.g. `selected` or `pressed`.
    pub fn update_theme_by_state(&mut self, force: bool) {
      
        self.update_internal_theme_by_state(force, &(self.current_node.clone()));
        crate::shell::CONSOLE.count_end("updates");
    }

    /// Update all properties for the theme.
    pub fn update_properties_by_theme(&mut self) {
        if !self.has::<crate::theme::Selector>("_selector") {
            return;
        }

        crate::shell::CONSOLE.count("updates");

        let selector = self.clone::<crate::theme::Selector>("_selector");

        if !selector.dirty {
            return;
        }

        if let Some(properties) = self._theme.properties(&selector) {
            for (key, value) in properties {
                match key.as_str() {
                    "foreground" | "background" | "border_brush" | "icon_brush" => self.update_brush(key, value),
                    "border_radius" | "font_size" | "icon_size" | "spacing" => self.update_f64(key, value),
                    "font" | "icon_font" => self.update_string(key, value),
                    "opacity" => self.update_f32(key, value),
                    "padding" | "border_width" => self.update_thickness_from_f64(key, value),
                    "padding_left" => self.update_thickness_part(key, "left", value),
                    "padding_top" => self.update_thickness_part(key, "top", value),
                    "padding_right" => self.update_thickness_part(key, "right", value),
                    "padding_bottom" => self.update_thickness_part(key, "bottom", value),
                    _ => {}
                }
            }
        }

        // if self.has::<Thickness>("border_width") {
        //     if let Some(border_width) = self._theme.property("border_width", &selector) {
        //         if let Ok(border_width) = border_width.into_rust::<f64>() {
        //             self.set("border_width", Thickness::from(border_width));
        //         }
        //     }
        // }

      

        // if let Some(mut padding) = self.try_clone::<Thickness>("padding") {
        //     if let Some(pad) = self.theme.uint("padding", &selector) {
        //         padding.set_thickness(pad as f64);
        //     }

        //     if let Some(left) = self.theme.uint("padding-left", &selector) {
        //         padding.set_left(left as f64);
        //     }

        //     if let Some(top) = self.theme.uint("padding-top", &selector) {
        //         padding.set_top(top as f64);
        //     }

        //     if let Some(right) = self.theme.uint("padding-right", &selector) {
        //         padding.set_right(right as f64);
        //     }

        //     if let Some(bottom) = self.theme.uint("padding-bottom", &selector) {
        //         padding.set_bottom(bottom as f64);
        //     }
        //     self.set::<Thickness>("padding", padding);
        // }

        // if let Some(mut constraint) = self.try_clone::<Constraint>("constraint") {
        //     if let Some(width) = self.theme.uint("width", &selector) {
        //         constraint.set_width(width as f64);
        //     }

        //     if let Some(height) = self.theme.uint("height", &selector) {
        //         constraint.set_height(height as f64);
        //     }

        //     if let Some(min_width) = self.theme.uint("min-width", &selector) {
        //         constraint.set_min_width(min_width as f64);
        //     }

        //     if let Some(min_height) = self.theme.uint("min-height", &selector) {
        //         constraint.set_min_height(min_height as f64);
        //     }

        //     if let Some(max_width) = self.theme.uint("max-width", &selector) {
        //         constraint.set_max_width(max_width as f64);
        //     }

        //     if let Some(max_height) = self.theme.uint("max-height", &selector) {
        //         constraint.set_max_height(max_height as f64);
        //     }

        //     self.set::<Constraint>("constraint", constraint);
        // }


        self.get_mut::<crate::theme::Selector>("_selector").dirty = false;
    }

    fn update_brush(&mut self, key: &str, value: &Value) {
        if self.has::<Brush>(key) {
            if let Ok(brush) = value.clone().into_rust::<String>() {
                self.set(key, Brush::from(brush));
            }
        }
    }

    fn update_string(&mut self, key: &str, value: &Value) {
        if self.has::<String>(key) {
            if let Ok(number) = value.clone().into_rust::<String>() {
                self.set(key, number);
            }
        }
    }

    fn update_f32(&mut self, key: &str, value: &Value) {
        if self.has::<f32>(key) {
            if let Ok(number) = value.clone().into_rust::<f32>() {
                self.set(key, number);
            }
        }
    }

    fn update_f64(&mut self, key: &str, value: &Value) {
        if self.has::<f64>(key) {
            if let Ok(number) = value.clone().into_rust::<f64>() {
                self.set(key, number);
            }
        }
    }

    fn update_thickness_from_f64(&mut self, key: &str, value: &Value) {
        if self.has::<Thickness>(key) {
            if let Ok(number) = value.clone().into_rust::<f64>() {
                self.set(key, Thickness::from(number));
            }
        }
    }

    fn update_thickness_part(&mut self, key: &str, direction: &str, value: &Value) {
        if self.has::<Thickness>(key) {
            if let Ok(number) = value.clone().into_rust::<f64>() {
                match direction {
                    "left" => self.get_mut::<Thickness>(key).set_left(number),
                    "top" =>  self.get_mut::<Thickness>(key).set_top(number),
                    "right" =>  self.get_mut::<Thickness>(key).set_right(number),
                    "bottom" =>  self.get_mut::<Thickness>(key).set_bottom(number),
                    _ => {}
                }
            }
        }
    }
}
