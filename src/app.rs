use std::marker::PhantomData;

use anyhow::Error;
use url::Url;

use crate::{clipboard::Clipboard, config::Config};

pub struct App<T: Clipboard> {
    config: Config,
    _marker: PhantomData<T>,
}

impl<T: Clipboard> App<T> {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            _marker: PhantomData,
        }
    }

    pub fn on_clipboard_update(&self) -> Result<(), Error> {
        let mut clipboard = T::try_open()?;

        if !clipboard.is_text_available() {
            // No text to process, ¯\_(ツ)_/¯
            return Ok(());
        }

        let text = clipboard.get_text()?;

        if let Some(new_text) = self.process_clipboard(text.as_str()) {
            clipboard.set_text(&new_text)?;
        }

        Ok(())
    }

    fn process_clipboard(&self, text: &str) -> Option<String> {
        if let Ok(mut url) = Url::parse(text) {
            for entry in self.config.values() {
                if url.domain() == Some(&entry.replace) {
                    url.set_host(Some(&entry.with)).unwrap();

                    return Some(url.to_string());
                }
            }
        }

        None
    }
}
