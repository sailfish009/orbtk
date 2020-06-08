use std::rc::Rc;

use dces::prelude::Entity;

use crate::{css_engine::*, event::EventHandler};

pub use self::build_context::*;
pub use self::context::*;
pub use self::registry::*;
pub use self::state::*;
pub use self::states_context::*;
pub use self::template::*;
pub use self::widget_container::*;

mod build_context;
mod context;
mod registry;
mod state;
mod states_context;
mod template;
mod widget_container;

/// Adds the given `pseudo_class` to the css selector of the given `widget`.
pub fn update_state(state: &str, flag: bool, widget: &mut WidgetContainer<'_>) {
    if let Some(selector) = widget.try_get_mut::<crate::theme::Selector>("_selector") {
        if flag && (selector.state.is_none() || selector.state.is_some() && !selector.state.as_ref().unwrap().eq(state)) {
            selector.set_state(state);
            selector.dirty = true;
        } else if !flag && selector.state.is_some() && selector.state.as_ref().unwrap().eq(state) {
            selector.state = None;
            selector.dirty = true;
        }
   
    }
}

/// Used to define the `parent_type`of a widget.
pub enum ParentType {
    /// No children could be added to the widget.
    None,

    /// Only one child could be added to the widget.
    Single,

    /// Multiple children could be added to the widget.
    Multi,
}

/// The `Widget` trait is used to define a new widget.
pub trait Widget: Template {
    /// Creates a new widget.
    fn create() -> Self;

    /// Builds the widget and returns the template of the widget.
    fn build(self, ctx: &mut BuildContext) -> Entity;

    /// Inerts a new event handler.
    fn insert_handler(self, handler: impl Into<Rc<dyn EventHandler>>) -> Self;

    /// Appends a child to the widget.
    fn child(self, child: Entity) -> Self;
}
