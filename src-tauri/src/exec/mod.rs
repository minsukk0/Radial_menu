pub mod app;
pub mod folder;
pub mod system;
pub mod terminal;

use radial_core::Item;
use crate::context::ContextValues;

pub fn execute_item(item: &Item, ctx: &ContextValues) -> Result<(), String> {
    match item {
        Item::App(a) => app::execute_app(a, ctx),
        Item::Terminal(t) => terminal::execute_terminal(t, ctx),
        Item::Folder(f) => folder::execute_folder(f, ctx),
        Item::System(s) => system::execute_system(s),
    }
}
