use std::any::Any;
use std::cell::Cell;
use std::rc::Rc;
use std::marker::PhantomData;

/// A token store
///
/// This struct allows you to store various values in a store
/// and access them back using the provided tokens.
pub struct Store {
    values: Vec<Option<(Box<Any>, Rc<Cell<bool>>)>>,
}

/// A token for accessing the store contents
pub struct Token<V> {
    id: usize,
    live: Rc<Cell<bool>>,
    _type: PhantomData<V>,
}

impl<V> Clone for Token<V> {
    fn clone(&self) -> Token<V> {
        Token {
            id: self.id,
            live: self.live.clone(),
            _type: PhantomData,
        }
    }
}

impl Store {
    /// Insert a new value in this store
    ///
    /// Returns a clonable token that you can later use to access this
    /// value.
    pub fn insert<V: Any + 'static>(&mut self, value: V) -> Token<V> {
        let boxed = Box::new(value) as Box<Any>;
        let live = Rc::new(Cell::new(true));
        {
            // artificial scope to make the borrow checker happy
            let empty_slot = self.values
                .iter_mut()
                .enumerate()
                .find(|&(_, ref s)| s.is_none());
            if let Some((id, slot)) = empty_slot {
                *slot = Some((boxed, live.clone()));
                return Token {
                    id: id,
                    live: live,
                    _type: PhantomData,
                };
            }
        }
        self.values.push(Some((boxed, live.clone())));
        Token {
            id: self.values.len() - 1,
            live: live,
            _type: PhantomData,
        }
    }

    /// Access value previously inserted in this store
    ///
    /// Panics if the provided token corresponds to a value that was removed.
    pub fn get<V: Any + 'static>(&self, token: &Token<V>) -> &V {
        if !token.live.get() {
            panic!("Attempted to access a state value that was already removed!");
        }
        self.values[token.id]
            .as_ref()
            .and_then(|t| t.0.downcast_ref::<V>())
            .unwrap()
    }

    /// Mutably access value previously inserted in this store
    ///
    /// Panics if the provided token corresponds to a value that was removed.
    pub fn get_mut<V: Any + 'static>(&mut self, token: &Token<V>) -> &mut V {
        if !token.live.get() {
            panic!("Attempted to access a state value that was already removed!");
        }
        self.values[token.id]
            .as_mut()
            .and_then(|t| t.0.downcast_mut::<V>())
            .unwrap()
    }

    /// Remove a value previously inserted in this store
    ///
    /// Panics if the provided token corresponds to a value that was already
    /// removed.
    pub fn remove<V: Any + 'static>(&mut self, token: Token<V>) -> V {
        if !token.live.get() {
            panic!("Attempted to remove a state value that was already removed!");
        }
        let (boxed, live) = self.values[token.id].take().unwrap();
        live.set(false);
        *boxed.downcast().unwrap()
    }
}
