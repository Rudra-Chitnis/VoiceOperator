pub struct ActionSpec {
    pub id: &'static str,
    pub aliases: &'static [&'static str],
    pub description: &'static str,
}

pub static ACTION_REGISTRY: &[ActionSpec] = &[
    ActionSpec {
        id: "open_notepad",
        aliases: &["open notepad", "start notepad", "launch notepad"],
        description: "Open Notepad",
    },
    ActionSpec {
        id: "open_explorer",
        aliases: &["open explorer", "start explorer", "launch explorer"],
        description: "Open File Explorer",
    },
    ActionSpec {
        id: "open_calculator",
        aliases: &["open calculator", "start calculator", "launch calculator"],
        description: "Open Calculator",
    },
    ActionSpec {
        id: "open_last_folder",
        aliases: &["open last folder", "go back to last folder"],
        description: "Open the previously opened folder",
    },
    ActionSpec {
        id: "help",
        aliases: &["help", "commands"],
        description: "Show available commands",
    },
    ActionSpec {
        id: "exit",
        aliases: &["exit", "quit", "stop"],
        description: "Exit VoiceOperator",
    },
];

pub fn find_action(input: &str) -> Option<&'static ActionSpec> {
    ACTION_REGISTRY.iter().find(|spec| {
        spec.aliases
            .iter()
            .any(|alias| input.eq_ignore_ascii_case(alias))
    })
}