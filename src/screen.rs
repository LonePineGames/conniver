use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ScreenLine {
  pub text: String,
  pub color: ScreenColor,
  pub importance: i32,
  pub order: i32,
  pub indent: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum ScreenColor {
  #[default]
  White,
  Green,
  Yellow,
  Cyan,
  Blue,
  Red,
  Count
}
