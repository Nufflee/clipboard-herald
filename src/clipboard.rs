use anyhow::Error;

pub trait Clipboard : Sized {
    fn try_open() -> Result<Self, Error>;
    fn is_text_available(&self) -> bool;
    fn get_text(&self) -> Result<String, Error>;
    fn set_text(&mut self, text: &str) -> Result<(), Error>;
}
