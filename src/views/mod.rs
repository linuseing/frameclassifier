use crate::app::GlobalState;

pub mod home;
pub mod label;
pub mod list;

pub trait View {
    fn render(&mut self, ctx: &egui::Context, app: &mut GlobalState) -> Option<Box<dyn View>>;
}
