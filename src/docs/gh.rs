use serde::{Deserialize};
use std::collections::HashMap;

// ----------- Repos - Start -----------

/** High level workflows associated to a repo and their general info 
* gh api repos/{owner}/{repo}/actions/workflows
*/

#[derive(Debug, Deserialize)]
pub struct RepoWorflowsOverview {
    pub total_count: u32,
    pub workflows: Vec<WorkflowMetaData>,
}

#[derive(Debug, Deserialize)]
pub struct WorkflowMetaData {
    pub id: u32,
    pub name: String,
    pub path: String,
    pub state: String,
    pub created_at: String,
    pub updated_at: String,
    pub url: String,
    pub html_url: String,
    pub badge_url: String,
}
// ----------- Repos - End -----------

/**
 * gh api repos/pr0methevs/gha-workflow-practice/branches --jq '.[].name'
 * 
 */

// -------- Workflows - Start ---------

#[derive(Debug, Deserialize)]
pub struct Workflow {
    pub name: String,
    pub on: WorkflowDispatch,
    pub jobs: serde_yaml::Value
}

#[derive(Debug, Deserialize)]
pub struct WorkflowDispatch {
    pub workflow_dispatch: Option<HashMap<String, WorkflowInputs>>,
}

#[derive(Debug, Deserialize)]
pub struct WorkflowInputs {
    pub inputs: HashMap<String, Input>,
}

#[derive(Debug, Deserialize)]
pub struct Input {
    pub description: Option<String>,
    pub required: Option<bool>,
    // pub default: Option<serde_yaml::Value>,
    pub default: Option<String>,
    #[serde(rename = "type")]
    pub input_type: Option<InputType>,
    // For the choice input type
    pub options: Option<Vec<String>>, 
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum InputType {
    String,
    Boolean,
    Choice,
    Environment,
}

impl Workflow {
    pub fn get_inputs(&self) -> Vec<String> {
        let mut inputs_list = Vec::new();

        if let Some(workflow_dispatch_map) = &self.on.workflow_dispatch {

            for (_key, workflow_dispatch) in workflow_dispatch_map {

                for (input_name, input) in &workflow_dispatch.inputs {
                    let mut input_str = format!("{}:", input_name);
                    if let Some(description) = &input.description {
                        input_str.push_str(&format!(" {}", description));
                    }
                    if let Some(required) = input.required {
                        input_str.push_str(&format!(" [required: {}]", required));
                    }
                    if let Some(default) = &input.default {
                        input_str.push_str(&format!(" [default: {}]", default));
                    }
                    if let Some(input_type) = &input.input_type {
                        input_str.push_str(&format!(" [type: {:?}]", input_type));
                    }
                    inputs_list.push(input_str);
                }
            }
        }

        inputs_list
    }
}

// -------- Workflows - End ---------

// Repos would come from config file ? 
// They would have branches associated 

pub struct RepoInterface {
    pub name: String,
    pub branches: Vec<String>,
    // pub workflows: Vec<Workflow>,
    pub workflows: HashMap<String, Vec<Workflow>>, 
}


/** Use the gh cli to get workflow file content
 * 
 * gh api "repos/{owner}/{repo}/contents/.github/workflows/{filename}?ref={branch}" --jq '.content' | base64 --decode
 */
pub fn get_workflow_config() {
    // use the above gh cli command to get the workflow file content
    

    
}