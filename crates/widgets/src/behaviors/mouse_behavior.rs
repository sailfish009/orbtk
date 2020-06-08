use crate::prelude::*;

#[derive(Debug, Copy, Clone)]
enum Action {
    Press(Mouse),
    Release(Mouse),
    Scroll(Point),
}

/// The `MouseBehaviorState` handles the `MouseBehavior` widget.
#[derive(Default, AsAny)]
pub struct MouseBehaviorState {
    action: Option<Action>,
    has_delta: bool,
}

impl MouseBehaviorState {
    fn action(&mut self, action: Action) {
        self.action = Some(action);
    }
}

impl State for MouseBehaviorState {
    fn update(&mut self, _: &mut Registry, ctx: &mut Context<'_>) {
    
        // println!("Mousebehavior");
        if self.action.is_none() || !ctx.widget().get::<bool>("enabled") {
            return;
        }

        // crate::shell::CONSOLE.time("mouse-behavior");

        match self.action.unwrap() {
            Action::Press(_) => {
                ctx.widget().set("pressed", true);
            }
            Action::Release(p) => {
                let pressed: bool = *ctx.widget().get("pressed");
                ctx.widget().set("pressed", false);

                if check_mouse_condition(Point::new(p.x, p.y), &ctx.widget()) && pressed {
                    let parent = ctx.entity_of_parent().unwrap();
                    ctx.push_event_by_entity(
                        ClickEvent {
                            position: Point::new(p.x, p.y),
                        },
                        parent,
                    )
                }
            }
            Action::Scroll(p) => {
                ctx.widget().set("position", p);
                self.has_delta = true;
            }
        };

        // crate::shell::CONSOLE.time("update_state");
        let target: Entity = (*ctx.widget().get::<u32>("target")).into();
        ctx.get_widget(target).update_theme_by_state(false);
        // crate::shell::CONSOLE.time_end("update_state");

        self.action = None;
        // crate::shell::CONSOLE.time_end("mouse-behavior");
    }

    fn update_post_layout(&mut self, _: &mut Registry, ctx: &mut Context<'_>) {
        if self.has_delta {
            ctx.widget().set("delta", Point::new(0.0, 0.0));
            self.has_delta = false;
        }
    }
}

widget!(
    /// The `MouseBehavior` widget is used to handle internal the pressed behavior of a widget.
    ///
    /// **CSS element:** `check-box`
    MouseBehavior<MouseBehaviorState>: MouseHandler {
        /// Sets or shares the target of the behavior.
        target: u32,

        /// Sets or shares the pressed property.
        pressed: bool,

        /// Sets or shares the (wheel, scroll) delta property.
        delta: Point
    }
);

impl Template for MouseBehavior {
    fn template(self, id: Entity, _: &mut BuildContext) -> Self {
        self.name("MouseBehavior")
            .delta(0.0)
            .pressed(false)
            .on_mouse_down(move |states, m| {
                states
                    .get_mut::<MouseBehaviorState>(id)
                    .action(Action::Press(m));
                false
            })
            .on_mouse_up(move |states, m| {
                states
                    .get_mut::<MouseBehaviorState>(id)
                    .action(Action::Release(m));
                false
            })
            .on_scroll(move |states, p| {
                states
                    .get_mut::<MouseBehaviorState>(id)
                    .action(Action::Scroll(p));
                false
            })
    }
}
