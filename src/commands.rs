#[derive(Debug)]
pub enum Intent {
    LaunchApplication {
        application: Application,
    },

    OpenPath {
        path: String,
    },

    ReopenLastPath,

    ShowHelp,

    Shutdown,
}

#[derive(Debug, Clone, Copy)]
pub enum Application {
    Notepad,
    Explorer,
    Calculator,
}