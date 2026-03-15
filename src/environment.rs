use std::borrow::Cow;

use crate::interpreter::LoxValue;

type EnvLayer<'src> = hashbrown::HashMap<Cow<'src, str>, LoxValue<'src>>;

#[derive(Debug, Clone)]
pub struct Environment<'src> {
    scopes: Vec<EnvLayer<'src>>,
}

impl Default for Environment<'_> {
    fn default() -> Self {
        Self {
            scopes: vec![Default::default()],
        }
    }
}

impl<'src> Environment<'src> {
    pub fn push_scope(&mut self) {
        self.scopes.push(Default::default());
    }

    pub fn pop_scope(&mut self) {
        if self.scopes.len() <= 1 {
            return;
        }
        self.scopes.pop();
    }

    pub fn get(&self, id: impl AsRef<str>) -> Option<&LoxValue<'src>> {
        let id = id.as_ref();
        self.scopes.iter().rev().find_map(|layer| layer.get(id))
    }

    pub fn define(&mut self, id: impl Into<Cow<'src, str>>, value: LoxValue<'src>) {
        self.scopes.last_mut().unwrap().insert(id.into(), value);
    }

    pub fn set(&mut self, id: impl AsRef<str>, value: LoxValue<'src>) -> bool {
        let id = id.as_ref();
        for layer in self.scopes.iter_mut().rev() {
            if let Some(old) = layer.get_mut(id) {
                *old = value;
                return true;
            }
        }
        false
    }
}
