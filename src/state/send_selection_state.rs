use std::path::PathBuf;

/// Canonical selected-send entry shared by send tab and the edit page.
#[derive(Clone, Debug)]
pub struct SendSelectionItem {
    pub path: PathBuf,
    pub name: String,
    pub size: u64,
    pub file_type: String,
    pub text_content: Option<String>,
}

impl SendSelectionItem {
    pub fn from_text(text: String) -> Self {
        let size = text.len() as u64;
        let name = text.clone();
        Self {
            path: PathBuf::from(name.clone()),
            name,
            size,
            file_type: "text/plain".to_string(),
            text_content: Some(text),
        }
    }
}

#[derive(Default)]
pub struct SendSelectionState {
    items: Vec<SendSelectionItem>,
}

impl SendSelectionState {
    pub fn items(&self) -> &[SendSelectionItem] {
        &self.items
    }

    pub fn total_size(&self) -> u64 {
        self.items.iter().map(|f| f.size).sum()
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    pub fn remove(&mut self, index: usize) {
        if index < self.items.len() {
            self.items.remove(index);
        }
    }

    pub fn add_text(&mut self, text: String) {
        self.items.push(SendSelectionItem::from_text(text));
    }

    pub fn update_text(&mut self, index: usize, text: String) {
        if let Some(item) = self.items.get_mut(index) {
            let size = text.len() as u64;
            item.name = text.clone();
            item.path = PathBuf::from(text.clone());
            item.file_type = "text/plain".to_string();
            item.size = size;
            item.text_content = Some(text);
        }
    }
}
