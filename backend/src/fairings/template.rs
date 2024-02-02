use std::path::PathBuf;

pub struct TemplateResolver {
    root: PathBuf,
}

impl TemplateResolver {
    pub fn new(root: &str) -> Self {
        let root = PathBuf::from(root);
        assert!(root.exists(), "Template directory does not exist");

        Self { root }
    }

    pub fn email_verification(&self) -> String {
        let path = self.root.join("email-verification.hbs");
        std::fs::read_to_string(path).expect("Failed to read email verification template")
    }
}
