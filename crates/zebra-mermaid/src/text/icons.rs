//! Host icons for one render. Ported from Pele `src/text/icons.ts`, with the module-level
//! resolver and cache moved into a value that lives for one render.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use super::entities::decode_entities;

/// Gives the SVG markup to draw inside a 24×24 box for an icon name such as `fa:fa-car`.
///
/// The markup is trusted: it is written into the output as given. A host resolves names
/// against icons it controls and returns `None` for names it does not know. The core renderer
/// never loads files or network resources for an icon.
pub trait IconResolver: Send + Sync {
    fn resolve(&self, name: &str) -> Option<String>;
}

impl<F: Fn(&str) -> Option<String> + Send + Sync> IconResolver for F {
    fn resolve(&self, name: &str) -> Option<String> {
        self(name)
    }
}

/// A fixed table of icons, for hosts and tests that know their icons up front.
#[derive(Debug, Clone, Default)]
pub struct IconTable {
    icons: HashMap<String, String>,
}

impl IconTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, name: impl Into<String>, markup: impl Into<String>) -> &mut Self {
        self.icons.insert(name.into(), markup.into());
        self
    }
}

impl IconResolver for IconTable {
    fn resolve(&self, name: &str) -> Option<String> {
        self.icons.get(name).cloned()
    }
}

/// The resolver of one render, with the answers it has given.
#[derive(Default)]
pub struct Icons {
    resolver: Option<Arc<dyn IconResolver>>,
    found: RefCell<HashMap<String, Rc<str>>>,
}

impl Icons {
    pub fn new(resolver: Option<Arc<dyn IconResolver>>) -> Self {
        Icons { resolver, found: RefCell::new(HashMap::new()) }
    }

    pub fn none() -> Self {
        Icons::default()
    }

    pub fn has_resolver(&self) -> bool {
        self.resolver.is_some()
    }

    /// The host's markup for `name`, or an empty string. The host sees the name with entity
    /// codes turned into their characters.
    pub fn markup(&self, name: &str) -> Rc<str> {
        let Some(resolver) = &self.resolver else {
            return Rc::from("");
        };
        if let Some(found) = self.found.borrow().get(name) {
            return found.clone();
        }
        let markup: Rc<str> = Rc::from(resolver.resolve(&decode_entities(name)).unwrap_or_default());
        self.found.borrow_mut().insert(name.to_string(), markup.clone());
        markup
    }
}
