use crate::Event;
use std::{cell::RefCell, rc::Rc};

#[derive(Clone)]
pub(crate) struct PendingEvents(Rc<RefCell<Vec<Event>>>);

impl PendingEvents {
    pub(crate) fn new() -> Self {
        Self(Rc::new(RefCell::new(Vec::new())))
    }

    pub(crate) fn push(&self, event: Event) {
        self.0.borrow_mut().push(event);
    }

    pub(crate) fn take(&self) -> Vec<Event> {
        self.0.take()
    }
}
