use std::{any::Any, collections::BTreeMap};

use dces::prelude::*;

use crate::{render::RenderContext2D, theming::*, tree::Tree, utils::*};

pub use self::absolute::*;
pub use self::fixed_size::*;
pub use self::grid::*;
pub use self::padding::*;
pub use self::popup::*;
pub use self::stack::*;
pub use self::text_selection::*;

mod absolute;
mod fixed_size;
mod grid;
mod padding;
mod popup;
mod stack;
mod text_selection;

/// A layout is used to dynamic order the children of a widget.
pub trait Layout: Any {
    // Measure all children before the arrangement.
    fn measure(
        &self,
        render_context_2_d: &mut RenderContext2D,
        entity: Entity,
        ecm: &mut EntityComponentManager<Tree, StringComponentStore>,
        layouts: &BTreeMap<Entity, Box<dyn Layout>>,
        theme: &Theme,
    ) -> DirtySize;

    /// Arranges and sizes the children.
    fn arrange(
        &self,
        render_context_2_d: &mut RenderContext2D,
        parent_size: (f64, f64),
        entity: Entity,
        ecm: &mut EntityComponentManager<Tree, StringComponentStore>,
        layouts: &BTreeMap<Entity, Box<dyn Layout>>,
        theme: &Theme,
    ) -> (f64, f64);
}

fn component<C: Component + Clone>(
    ecm: &mut EntityComponentManager<Tree, StringComponentStore>,
    entity: Entity,
    component: &str,
) -> C {
    ecm.component_store()
        .get::<C>(component, entity)
        .unwrap()
        .clone()
}

fn try_component<C: Component + Clone>(
    ecm: &mut EntityComponentManager<Tree, StringComponentStore>,
    entity: Entity,
    component: &str,
) -> Option<C> {
    if let Ok(c) = ecm.component_store().get::<C>(component, entity) {
        return Some(c.clone());
    }

    None
}

fn component_or_default<C: Component + Clone + Default>(
    ecm: &mut EntityComponentManager<Tree, StringComponentStore>,
    entity: Entity,
    component: &str,
) -> C {
    ecm.component_store()
        .get::<C>(component, entity)
        .map(Clone::clone)
        .unwrap_or_default()
}

fn component_try_mut<'a, C: Component>(
    ecm: &'a mut EntityComponentManager<Tree, StringComponentStore>,
    entity: Entity,
    component: &str,
) -> Option<&'a mut C> {
    ecm.component_store_mut()
        .get_mut::<C>(component, entity)
        .ok()
}
