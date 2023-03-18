use std::{cell::RefCell, collections::HashMap};

use web_sys::Document;

use crate::ListenerId;

thread_local! {
    static CUSTOM_DOCUMENTS: RefCell<HashMap<ListenerId, Document>> = RefCell::default();
}

pub fn delete_document(id: ListenerId) {
    CUSTOM_DOCUMENTS.with(|documents| {
        documents.borrow_mut().remove(&id);
    });
}

pub fn register_document(id: ListenerId, document: Document) {
    CUSTOM_DOCUMENTS.with(|documents| {
        documents.borrow_mut().insert(id, document);
    });
}

pub fn get_document(id: ListenerId) -> Option<Document> {
    CUSTOM_DOCUMENTS.with(|documents| documents.borrow().get(&id).cloned())
}
