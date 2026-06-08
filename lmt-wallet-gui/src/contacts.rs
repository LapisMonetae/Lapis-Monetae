use crate::config::Contact;
use crate::validators;

pub struct ContactsManager {
    pub contacts: Vec<Contact>,
    pub show_dialog: bool,
    pub validation_error: String,
    pub network: String,
    pub edit_name: String,
    pub edit_address: String,
    pub edit_note: String,
    pub editing_index: Option<usize>,
    pub search: String,
}

impl ContactsManager {
    pub fn new(contacts: Vec<Contact>, network: String) -> Self {
        Self {
            contacts,
            show_dialog: false,
            validation_error: String::new(),
            network,
            edit_name: String::new(),
            edit_address: String::new(),
            edit_note: String::new(),
            editing_index: None,
            search: String::new(),
        }
    }

    pub fn open_add(&mut self) {
        self.edit_name.clear();
        self.edit_address.clear();
        self.edit_note.clear();
        self.validation_error.clear();
        self.editing_index = None;
        self.show_dialog = true;
    }

    pub fn open_edit(&mut self, index: usize) {
        if let Some(c) = self.contacts.get(index) {
            self.edit_name = c.name.clone();
            self.edit_address = c.address.clone();
            self.edit_note = c.note.clone();
            self.editing_index = Some(index);
            self.show_dialog = true;
        }
    }

    pub fn save_contact(&mut self) -> bool {
        self.validation_error.clear();
        if self.edit_name.trim().is_empty() {
            self.validation_error = "Name is required".into();
            return false;
        }
        if self.edit_address.trim().is_empty() {
            self.validation_error = "Address is required".into();
            return false;
        }
        // Validate LMT address
        if let Err(e) = validators::validate_address(&self.edit_address, &self.network) {
            self.validation_error = format!("Invalid address: {e}");
            return false;
        }
        let contact = Contact {
            name: self.edit_name.trim().to_string(),
            address: self.edit_address.trim().to_string(),
            note: self.edit_note.trim().to_string(),
        };
        if let Some(idx) = self.editing_index {
            if idx < self.contacts.len() {
                self.contacts[idx] = contact;
            }
        } else {
            if self.contacts.iter().any(|c| c.address == contact.address) {
                self.validation_error = "Contact with this address already exists".into();
                return false;
            }
            self.contacts.push(contact);
        }
        self.show_dialog = false;
        true
    }

    pub fn remove(&mut self, index: usize) {
        if index < self.contacts.len() {
            self.contacts.remove(index);
        }
    }

    pub fn filtered(&self) -> Vec<(usize, &Contact)> {
        let q = self.search.to_lowercase();
        self.contacts
            .iter()
            .enumerate()
            .filter(|(_, c)| q.is_empty() || c.name.to_lowercase().contains(&q) || c.address.to_lowercase().contains(&q))
            .collect()
    }
}
