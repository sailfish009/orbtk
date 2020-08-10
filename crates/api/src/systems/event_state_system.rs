use std::{cell::RefCell, rc::Rc};

use dces::prelude::*;

use crate::{prelude::*, render::RenderContext2D, theming::Theme, tree::Tree, utils::*};

/// The `EventStateSystem` pops events from the event queue and delegates the events to the corresponding event handlers of the widgets and updates the states.
#[derive(Constructor)]
pub struct EventStateSystem {
    context_provider: ContextProvider,
    registry: Rc<RefCell<Registry>>,
}

impl EventStateSystem {
    // Remove all objects of a widget.
    fn remove_widget(
        &self,
        entity: Entity,
        theme: &Theme,
        ecm: &mut EntityComponentManager<Tree, StringComponentStore>,
        render_context: &mut RenderContext2D,
    ) {
        {
            let registry = &mut self.registry.borrow_mut();

            let mut ctx = Context::new(
                (entity, ecm),
                &theme,
                &self.context_provider,
                render_context,
            );

            if let Some(state) = self.context_provider.states.borrow_mut().get_mut(&entity) {
                state.cleanup(registry, &mut ctx);
            }

            drop(ctx);
        }
        self.context_provider.states.borrow_mut().remove(&entity);

        ecm.remove_entity(entity);
        self.context_provider.layouts.borrow_mut().remove(&entity);
        self.context_provider
            .render_objects
            .borrow_mut()
            .remove(&entity);
        self.context_provider
            .handler_map
            .borrow_mut()
            .remove(&entity);
    }

    fn process_direct(
        &self,
        event: &EventBox,
        ecm: &mut EntityComponentManager<Tree, StringComponentStore>,
    ) -> bool {
        // skip all direct events on first run
        if self.context_provider.first_run.get() {
            return false;
        }

        if event.strategy == EventStrategy::Direct {
            if let Some(handlers) = self
                .context_provider
                .handler_map
                .borrow()
                .get(&event.source)
            {
                handlers.iter().any(|handler| {
                    handler.handle_event(
                        &mut StatesContext::new(
                            &mut *self.context_provider.states.borrow_mut(),
                            ecm,
                        ),
                        &event,
                    )
                });
                return true;
            }
        }

        false
    }

    fn process_bottom_up_event(
        &self,
        mouse_position: Point,
        event: &EventBox,
        ecm: &mut EntityComponentManager<Tree, StringComponentStore>,
    ) -> bool {
        let mut matching_nodes = vec![];
        let mut update = false;

        let mut current_node = event.source;
        let root = ecm.entity_store().root();
        let mut disabled_parents = vec![];

        let theme = ecm
            .component_store()
            .get::<Global>("global", root)
            .unwrap()
            .theme
            .clone();

        // global key handling
        if let Ok(event) = event.downcast_ref::<KeyDownEvent>() {
            if let Ok(global) = ecm.component_store_mut().get_mut::<Global>("global", root) {
                // Set this value on the keyboard state
                global.keyboard_state.set_key_state(event.event.key, true);
            }
        }

        if let Ok(event) = event.downcast_ref::<KeyUpEvent>() {
            if let Ok(global) = ecm.component_store_mut().get_mut::<Global>("global", root) {
                // Set this value on the keyboard state
                global.keyboard_state.set_key_state(event.event.key, false);
            }
        }

        let mut unknown_event = true;
        let mut clipped_parent = vec![];

        loop {
            if !disabled_parents.is_empty() {
                if let Some(parent) = ecm.entity_store().parent[&current_node] {
                    if disabled_parents.contains(&parent) {
                        disabled_parents.push(current_node);
                    } else {
                        disabled_parents.clear();
                    }
                }
            }
            if let Ok(enabled) = ecm.component_store().get::<bool>("enabled", current_node) {
                if !enabled {
                    disabled_parents.push(current_node);
                }
            }

            if let Ok(visibility) = ecm
                .component_store()
                .get::<Visibility>("visibility", current_node)
            {
                if *visibility != Visibility::Visible {
                    disabled_parents.push(current_node);
                }
            }

            if disabled_parents.is_empty() {
                let mut has_handler = false;
                if let Some(handlers) = self
                    .context_provider
                    .handler_map
                    .borrow()
                    .get(&current_node)
                {
                    if handlers.iter().any(|handler| handler.handles_event(event)) {
                        has_handler = true;
                    }
                }
                if let Some(cp) = clipped_parent.last() {
                    if ecm.entity_store().parent[&current_node] == Some(*cp) {
                        clipped_parent.push(current_node);
                    } else {
                        clipped_parent.pop();
                    }
                }

                // scroll handling
                if event.downcast_ref::<ScrollEvent>().is_ok() {
                    if check_mouse_condition(
                        mouse_position,
                        &WidgetContainer::new(
                            current_node,
                            ecm,
                            &theme,
                            Some(&self.context_provider.event_queue),
                        ),
                    ) && has_handler
                    {
                        matching_nodes.push(current_node);
                    }
                    unknown_event = false;
                }
                // click handling
                if let Ok(event) = event.downcast_ref::<ClickEvent>() {
                    if check_mouse_condition(
                        event.position,
                        &WidgetContainer::new(
                            current_node,
                            ecm,
                            &theme,
                            Some(&self.context_provider.event_queue),
                        ),
                    ) {
                        let mut add = true;
                        if let Some(op) = clipped_parent.get(0) {
                            if !check_mouse_condition(
                                event.position,
                                &WidgetContainer::new(
                                    *op,
                                    ecm,
                                    &theme,
                                    Some(&self.context_provider.event_queue),
                                ),
                            ) {
                                add = false;
                            }
                        }
                        if add && has_handler {
                            matching_nodes.push(current_node);
                        }
                    }
                    unknown_event = false;
                }
                // mouse down handling
                if let Ok(event) = event.downcast_ref::<MouseDownEvent>() {
                    if check_mouse_condition(
                        event.position,
                        &WidgetContainer::new(
                            current_node,
                            ecm,
                            &theme,
                            Some(&self.context_provider.event_queue),
                        ),
                    ) {
                        let mut add = true;
                        if let Some(op) = clipped_parent.get(0) {
                            // todo: improve check path if exists
                            if !check_mouse_condition(
                                event.position,
                                &WidgetContainer::new(
                                    *op,
                                    ecm,
                                    &theme,
                                    Some(&self.context_provider.event_queue),
                                ),
                            ) && has_handler
                            {
                                add = false;
                            }
                        }
                        if add {
                            matching_nodes.push(current_node);
                        }
                    }
                    unknown_event = false;
                }
                // mouse move handling
                if let Ok(event) = event.downcast_ref::<MouseMoveEvent>() {
                    if check_mouse_condition(
                        event.position,
                        &WidgetContainer::new(
                            current_node,
                            ecm,
                            &theme,
                            Some(&self.context_provider.event_queue),
                        ),
                    ) {
                        let mut add = true;
                        if let Some(op) = clipped_parent.get(0) {
                            // todo: improve check path if exists
                            if !check_mouse_condition(
                                event.position,
                                &WidgetContainer::new(
                                    *op,
                                    ecm,
                                    &theme,
                                    Some(&self.context_provider.event_queue),
                                ),
                            ) {
                                add = false;
                            }
                        }
                        if add && has_handler {
                            matching_nodes.push(current_node);
                        }
                    }
                    unknown_event = false;
                }

                if unknown_event
                    && *WidgetContainer::new(
                        current_node,
                        ecm,
                        &theme,
                        Some(&self.context_provider.event_queue),
                    )
                    .get::<bool>("enabled")
                {
                    if has_handler {
                        matching_nodes.push(current_node);
                    }
                }
                if let Ok(clip) = ecm.component_store().get::<bool>("clip", current_node) {
                    if *clip {
                        clipped_parent.clear();
                        clipped_parent.push(current_node);
                    }
                }
            }

            let mut it = ecm.entity_store().start_node(current_node).into_iter();
            it.next();

            if let Some(node) = it.next() {
                current_node = node;
            } else {
                break;
            }
        }

        let mut handled = false;

        for node in matching_nodes.iter().rev() {
            if let Some(handlers) = self.context_provider.handler_map.borrow().get(node) {
                handled = handlers.iter().any(|handler| {
                    handler.handle_event(
                        &mut StatesContext::new(
                            &mut *self.context_provider.states.borrow_mut(),
                            ecm,
                        ),
                        event,
                    )
                });

                update = true;
            }

            if handled {
                break;
            }
        }

        update
    }
}

impl System<Tree, StringComponentStore, RenderContext2D> for EventStateSystem {
    fn run_with_context(
        &self,
        ecm: &mut EntityComponentManager<Tree, StringComponentStore>,
        render_context: &mut RenderContext2D,
    ) {
        let mut update = false;

        loop {
            {
                let mouse_position = self.context_provider.mouse_position.get();
                for event in self.context_provider.event_queue.borrow_mut().into_iter() {
                    if let Ok(event) = event.downcast_ref::<SystemEvent>() {
                        match event {
                            SystemEvent::Quit => {
                                // todo send close shell request
                                return;
                            }
                        }
                    }

                    match event.strategy {
                        EventStrategy::Direct => {
                            if event.strategy == EventStrategy::Direct {
                                update = self.process_direct(&event, ecm) || update;
                            }
                        }
                        // EventStrategy::TopDown => {
                        //     self.process_top_down_event(&event, ecm);
                        // }
                        EventStrategy::BottomUp => {
                            let should_update =
                                self.process_bottom_up_event(mouse_position, &event, ecm);
                            update = update || should_update;
                        }
                    }
                }
            }

            // handle states

            // crate::shell::CONSOLE.time("update-time:");

            let root = ecm.entity_store().root();

            let theme = ecm
                .component_store()
                .get::<Global>("global", root)
                .unwrap()
                .theme
                .clone();

            let mut remove_widget_list: Vec<Entity> = vec![];

            let mut dirty_index = 0;

            loop {
                if dirty_index
                    >= ecm
                        .component_store()
                        .get::<Vec<Entity>>("dirty_widgets", root)
                        .unwrap()
                        .len()
                {
                    break;
                }

                let mut skip = false;

                let widget = *ecm
                    .component_store()
                    .get::<Vec<Entity>>("dirty_widgets", root)
                    .unwrap()
                    .get(dirty_index)
                    .unwrap();

                if !self.context_provider.states.borrow().contains_key(&widget) {
                    skip = true;
                }

                let mut keys = vec![];

                if !skip {
                    {
                        let registry = &mut self.registry.borrow_mut();

                        let mut ctx = Context::new(
                            (widget, ecm),
                            &theme,
                            &self.context_provider,
                            render_context,
                        );

                        if let Some(state) =
                            self.context_provider.states.borrow_mut().get_mut(&widget)
                        {
                            state.update(registry, &mut ctx);
                        }

                        keys.append(&mut ctx.new_states_keys());

                        remove_widget_list.append(ctx.remove_widget_list());
                        drop(ctx);

                        for key in keys {
                            let mut ctx = Context::new(
                                (key, ecm),
                                &theme,
                                &self.context_provider,
                                render_context,
                            );
                            if let Some(state) =
                                self.context_provider.states.borrow_mut().get_mut(&key)
                            {
                                state.init(registry, &mut ctx);
                            }

                            drop(ctx);
                        }
                    }

                    for remove_widget in remove_widget_list.pop() {
                        let mut children = vec![];
                        get_all_children(&mut children, remove_widget, ecm.entity_store());

                        // remove children of target widget.
                        for entity in children.iter().rev() {
                            self.remove_widget(*entity, &theme, ecm, render_context);
                        }

                        // remove target widget
                        self.remove_widget(remove_widget, &theme, ecm, render_context);
                    }
                }

                dirty_index += 1;
            }

            // crate::shell::CONSOLE.time_end("update-time:");

            if self.context_provider.event_queue.borrow().is_empty() {
                break;
            }
        }
    }
}
