pub mod startup;
pub mod console;
pub mod help;
pub mod about;

#[derive(PartialEq, Clone)]
pub enum MainScreen {
    Console,
    Help,
    About
}
