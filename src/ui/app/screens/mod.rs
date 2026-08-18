pub mod startup;
pub mod console;
pub mod help;
pub mod about;

#[derive(PartialEq)]
pub enum MainScreen {
    Console,
    Help,
    About
}
