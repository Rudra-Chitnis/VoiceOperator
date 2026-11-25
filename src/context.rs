#[derive(Default, Debug)]
pub struct SessionContext {
    pub last_folder: Option<String>,
    pub last_application: Option<String>,
}

impl SessionContext {
    pub fn remember_folder(&mut self, path: String) {
        self.last_folder = Some(path);
    }

    pub fn remember_application(&mut self, app: String) {
        self.last_application = Some(app);
    }
}