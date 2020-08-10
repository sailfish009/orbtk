#![crate_name = "orbtk"]
#![crate_type = "lib"]

// pub use orbtk_api as api;
// pub use orbtk_proc_macros as proc_macros;
// pub use orbtk_render as render;
// pub use orbtk_shell as shell;
// pub use orbtk_theme as theme;
// pub use orbtk_theming as theming;
// pub use orbtk_tree as tree;
// pub use orbtk_utils as utils;
// pub use orbtk_widgets as widgets;
pub use orbtk_shell::prelude::initialize;

pub mod api {
    pub use orbtk_api::*;
}

pub mod proc_macros {
    pub use orbtk_proc_macros::*;
}

pub mod render {
    pub use orbtk_render::*;
}

pub mod shell {
    pub use orbtk_shell::*;
}

pub mod theming {
    pub use orbtk_theming::*;
}

pub mod theme {
    pub use orbtk_theme::*;
}

pub mod tree {
    pub use orbtk_tree::*;
}

pub mod utils {
    pub use orbtk_utils::*;
}

pub mod widgets {
    pub use orbtk_widgets::*;
}

pub mod prelude;
