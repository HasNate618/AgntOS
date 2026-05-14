// Model routing by task class.

pub enum TaskClass {
    GeneralChat,
    OsPlanning,
    ConfigEditing,
    CodeEditing,
    LogAnalysis,
    PrivateLocal,
}

pub struct ModelRoute {
    pub task: TaskClass,
    pub provider: String,
    pub model: String,
}
