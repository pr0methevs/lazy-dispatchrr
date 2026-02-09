
#[derive(Debug, Default)]
pub struct Repo {
    pub name: String,
    pub branches: Vec<String>,
    pub workflows: Vec<String>,
}

#[derive(Debug, Default, Clone)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub inputs: Vec<String>,
    // other metadata
}

#[derive(Debug, Clone)]
pub struct InputField {
    pub name: String,
    pub description: String,
    pub input_type: String,   // "string", "boolean", "choice", "environment"
    pub required: bool,
    pub default_value: String,
    pub options: Vec<String>,  // for choice type
    pub value: String,         // user-entered value
}
