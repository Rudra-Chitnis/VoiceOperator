#[derive(Debug)]
pub enum Intent {
    LaunchApplication {
        application: Application,
    },

    LaunchDiscoveredApp {
        name: String,
        target: String,
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