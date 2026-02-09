# GH CLI

## List all workflows on a given {owner}/{repo}

**List all workflows with their details in JSON**

```bash
gh api repos/{owner}/{repo}/actions/workflows

gh api repos/pr0methevs/gha-workflow-practice/actions/workflows
```


Result

```json
{
  "total_count": 1,
  "workflows": [
    {
      "id": 221940667,
      "node_id": "W_kwDOQ2Ya6c4NOou7",
      "name": "Input Types Showcase",
      "path": ".github/workflows/basic_workflow.yml",
      "state": "active",
      "created_at": "2026-01-08T20:35:32.000-05:00",
      "updated_at": "2026-01-08T20:35:32.000-05:00",
      "url": "https://api.github.com/repos/pr0methevs/gha-workflow-practice/actions/workflows/221940667",
      "html_url": "https://github.com/pr0methevs/gha-workflow-practice/blob/master/.github/workflows/basic_workflow.yml",
      "badge_url": "https://github.com/pr0methevs/gha-workflow-practice/workflows/Input%20Types%20Showcase/badge.svg"
    }
  ]
}
```

**Get Workflow by filename**

```bash
gh api repos/{owner}/{repo}/actions/workflows/{filename}

gh api repos/pr0methevs/gha-workflow-practice/actions/workflows/ basic_workflow.yml
```

Result

```json
{
  "id": 221940667,
  "node_id": "W_kwDOQ2Ya6c4NOou7",
  "name": "Input Types Showcase",
  "path": ".github/workflows/basic_workflow.yml",
  "state": "active",
  "created_at": "2026-01-08T20:35:32.000-05:00",
  "updated_at": "2026-01-08T20:35:32.000-05:00",
  "url": "https://api.github.com/repos/pr0methevs/gha-workflow-practice/actions/workflows/221940667",
  "html_url": "https://github.com/pr0methevs/gha-workflow-practice/blob/master/.github/workflows/basic_workflow.yml",
  "badge_url": "https://github.com/pr0methevs/gha-workflow-practice/workflows/Input%20Types%20Showcase/badge.svg"
}
```


## Get Workflows' Inputs

```bash
gh workflow view ${workflowName | id} -R ${repo} --ref ${branch} --yaml

gh workflow view "221940667" -R pr0methevs/gha-workflow-practice --ref master --yaml

# gh api equivalent
gh api "repos/{owner}/{repo}/contents/.github/workflows/{filename}?ref={branch}" --jq '.content' | base64 --decode
gh api "repos/pr0methevs/gha-workflow-practice/contents/.github/workflows/basic_workflow.yml?ref=master" --jq '.content' | base64 --decode
```

Result

```yaml
Input Types Showcase - basic_workflow.yml
ID: 221940667

# GitHub Actions Workflow - Input Types Showcase
# This workflow demonstrates all possible workflow_dispatch input types
# Run manually from Actions tab -> "Input Types Showcase" -> "Run workflow"

name: Input Types Showcase

on:
  workflow_dispatch:
    inputs:
      # STRING INPUT - Free-form text input
      string_input:
        description: 'A string input (free-form text)'
        required: true
        type: string
        default: 'Hello, World!'

      # NUMBER INPUT - Numeric value
      number_input:
        description: 'A number input (integer or float)'
        required: true
        type: number
        default: 42

      # BOOLEAN INPUT - True/False checkbox
      boolean_input:
        description: 'A boolean input (checkbox)'
        required: true
        type: boolean
        default: true

      # CHOICE INPUT - Dropdown selection
      choice_input:
        description: 'A choice input (dropdown selection)'
        required: true
        type: choice
        default: 'option-b'
        options:
          - 'option-a'
          - 'option-b'
          - 'option-c'
          - 'option-d'

      # ENVIRONMENT INPUT - Environment selector
      environment_input:
        description: 'An environment input (environment selector)'
        required: true
        type: environment

```


## List Worfklows for a Specific Repo

```bash
gh workflow list -R {owner}/{repo}
gh workflow list -R pr0methevs/gha-workflow-practice
```

Result

```
NAME                  STATE   ID       
Input Types Showcase  active  221940667
```


## List Recent Workflow Runs

```bash
gh run list -R {owner}/{repo}

gh run list -R pr0methevs/gha-workflow-practice
```

## Pick a Specific Workflow and Retrieve Logs

```bash
gh run view -R {owner}/{repo}

gh run view -R pr0methevs/gha-workflow-practice
```


## View "Full" Job/Step Logs 

Each Job/Step has a associated ID 

```bash
gh run view -R pr0methevs/gha-workflow-practice --log --job=62158787339
```


## Current Start Implementation

```bash
# Get workflow metadata (REST)
gh api repos/$OWNER/$REPO/actions/workflows --jq '.workflows[] | {id, name, state, html_url, path}'

# Get branches and YAML content (GraphQL)
gh api graphql -f query='
  query($owner: String!, $name: String!) {
    repository(owner: $owner, name: $name) {
      refs(first: 30, refPrefix: "refs/heads/") {
        nodes { name }
      }
      object(expression: "HEAD:.github/workflows/") {
        ... on Tree {
          entries {
            name
            object { ... on Blob { text } }
          }
        }
      }
    }
  }' -F owner="$OWNER" -F name="$REPO"
```

Response

```json
gh api graphql -f query='
  query($owner: String!, $name: String!) {
    repository(owner: $owner, name: $name) {
      refs(first: 10, refPrefix: "refs/heads/") {
        nodes { name }
      }
      object(expression: "HEAD:.github/workflows/") {
        ... on Tree {
          entries {
            name
            object {
              ... on Blob { text }
            }
          }
        }
      }
    }
  }' -F owner="pr0methevs" -F name="gha-workflow-practice"
{
  "data": {
    "repository": {
      "refs": {
        "nodes": [
          {
            "name": "dev"
          },
          {
            "name": "master"
          }
        ]
      },
      "object": {
        "entries": [
          {
            "name": "basic_workflow.yml",
            "object": {
              "text": "# GitHub Actions Workflow - Input Types Showcase\n# This workflow demonstrates all possible workflow_dispatch input types\n# Run manually from Actions tab -> \"Input Types Showcase\" -> \"Run workflow\"\n\nname: Input Types Showcase\n\non:\n  workflow_dispatch:\n    inputs:\n      # STRING INPUT - Free-form text input\n      string_input:\n        description: 'A string input (free-form text)'\n        required: true\n        type: string\n        default: 'Hello, World!'\n\n      # NUMBER INPUT - Numeric value\n      number_input:\n        description: 'A number input (integer or float)'\n        required: true\n        type: number\n        default: 42\n\n      # BOOLEAN INPUT - True/False checkbox\n      boolean_input:\n        description: 'A boolean input (checkbox)'\n        required: true\n        type: boolean\n        default: true\n\n      # CHOICE INPUT - Dropdown selection\n      choice_input:\n        description: 'A choice input (dropdown selection)'\n        required: true\n        type: choice\n        default: 'option-b'\n        options:\n          - 'option-a'\n          - 'option-b'\n          - 'option-c'\n          - 'option-d'\n\n      # ENVIRONMENT INPUT - Environment selector\n      environment_input:\n        description: 'An environment input (environment selector)'\n        required: true\n        type: environment\n\njobs:\n  # ============================================\n  # JOB 1: String Input Demonstration\n  # ============================================\n  string-job:\n    name: String Input Job\n    runs-on: ubuntu-latest\n    steps:\n      - name: Display String Input\n        run: |\n          echo \"============================================\"\n          echo \"STRING INPUT DEMONSTRATION\"\n          echo \"============================================\"\n          echo \"Raw value: ${{ inputs.string_input }}\"\n          echo \"Type: string\"\n          echo \"============================================\"\n\n      - name: Process String Input\n        run: |\n          INPUT_VALUE=\"${{ inputs.string_input }}\"\n          echo \"String length: ${#INPUT_VALUE}\"\n          echo \"Uppercase: ${INPUT_VALUE^^}\"\n          echo \"Lowercase: ${INPUT_VALUE,,}\"\n\n  # ============================================\n  # JOB 2: Number Input Demonstration\n  # ============================================\n  number-job:\n    name: Number Input Job\n    runs-on: ubuntu-latest\n    steps:\n      - name: Display Number Input\n        run: |\n          echo \"============================================\"\n          echo \"NUMBER INPUT DEMONSTRATION\"\n          echo \"============================================\"\n          echo \"Raw value: ${{ inputs.number_input }}\"\n          echo \"Type: number\"\n          echo \"============================================\"\n\n      - name: Perform Arithmetic Operations\n        run: |\n          NUM=${{ inputs.number_input }}\n          echo \"Original number: $NUM\"\n          echo \"Doubled: $((NUM * 2))\"\n          echo \"Squared: $((NUM * NUM))\"\n          echo \"Plus 100: $((NUM + 100))\"\n\n  # ============================================\n  # JOB 3: Boolean Input Demonstration\n  # ============================================\n  boolean-job:\n    name: Boolean Input Job\n    runs-on: ubuntu-latest\n    steps:\n      - name: Display Boolean Input\n        run: |\n          echo \"============================================\"\n          echo \"BOOLEAN INPUT DEMONSTRATION\"\n          echo \"============================================\"\n          echo \"Raw value: ${{ inputs.boolean_input }}\"\n          echo \"Type: boolean\"\n          echo \"============================================\"\n\n      - name: Conditional Logic Based on Boolean\n        run: |\n          if [ \"${{ inputs.boolean_input }}\" == \"true\" ]; then\n            echo \"✅ Boolean is TRUE - executing true branch\"\n            echo \"This step runs when the checkbox is checked\"\n          else\n            echo \"❌ Boolean is FALSE - executing false branch\"\n            echo \"This step runs when the checkbox is unchecked\"\n          fi\n\n  # ============================================\n  # JOB 4: Choice Input Demonstration\n  # ============================================\n  choice-job:\n    name: Choice Input Job\n    runs-on: ubuntu-latest\n    steps:\n      - name: Display Choice Input\n        run: |\n          echo \"============================================\"\n          echo \"CHOICE INPUT DEMONSTRATION\"\n          echo \"============================================\"\n          echo \"Selected value: ${{ inputs.choice_input }}\"\n          echo \"Type: choice\"\n          echo \"Available options: option-a, option-b, option-c, option-d\"\n          echo \"============================================\"\n\n      - name: Handle Different Choices\n        run: |\n          case \"${{ inputs.choice_input }}\" in\n            \"option-a\")\n              echo \"You selected Option A\"\n              echo \"Performing Option A specific actions...\"\n              ;;\n            \"option-b\")\n              echo \"You selected Option B\"\n              echo \"Performing Option B specific actions...\"\n              ;;\n            \"option-c\")\n              echo \"You selected Option C\"\n              echo \"Performing Option C specific actions...\"\n              ;;\n            \"option-d\")\n              echo \"You selected Option D\"\n              echo \"Performing Option D specific actions...\"\n              ;;\n            *)\n              echo \"Unknown option selected\"\n              ;;\n          esac\n\n  # ============================================\n  # JOB 5: Environment Input Demonstration\n  # ============================================\n  environment-job:\n    name: 🌍 Environment Input Job\n    runs-on: ubuntu-latest\n    environment: ${{ inputs.environment_input }}\n    steps:\n      - name: Display Environment Input\n        run: |\n          echo \"============================================\"\n          echo \"ENVIRONMENT INPUT DEMONSTRATION\"\n          echo \"============================================\"\n          echo \"Selected environment: ${{ inputs.environment_input }}\"\n          echo \"Type: environment\"\n          echo \"============================================\"\n\n      - name: Show Environment Context\n        run: |\n          echo \"Environment Name: ${{ inputs.environment_input }}\"\n          echo \"Environment URL: ${{ vars.ENVIRONMENT_URL || 'Not configured' }}\"\n          echo \"\"\n          echo \"Note: Environment inputs integrate with GitHub Environments\"\n          echo \"You can use environment-specific secrets and variables\"\n          echo \"Protection rules and reviewers can be configured per environment\"\n\n  # ============================================\n  # JOB 6: Summary Job (Aggregates All Inputs)\n  # ============================================\n  summary-job:\n    name: Summary Job\n    runs-on: ubuntu-latest\n    needs: [string-job, number-job, boolean-job, choice-job, environment-job]\n    steps:\n      - name: Generate Input Summary\n        run: |\n          echo \"============================================\"\n          echo \"       ALL INPUTS SUMMARY\"\n          echo \"============================================\"\n          echo \"\"\n          echo \"String Input:      ${{ inputs.string_input }}\"\n          echo \"Number Input:      ${{ inputs.number_input }}\"\n          echo \"Boolean Input:     ${{ inputs.boolean_input }}\"\n          echo \"Choice Input:      ${{ inputs.choice_input }}\"\n          echo \"Environment Input: ${{ inputs.environment_input }}\"\n          echo \"\"\n          echo \"============================================\"\n\n      - name: Create Job Summary\n        run: |\n          echo \"## 🎯 Workflow Dispatch Input Showcase Results\" >> $GITHUB_STEP_SUMMARY\n          echo \"\" >> $GITHUB_STEP_SUMMARY\n          echo \"| Input Type | Value |\" >> $GITHUB_STEP_SUMMARY\n          echo \"|------------|-------|\" >> $GITHUB_STEP_SUMMARY\n          echo \"| String | \\`${{ inputs.string_input }}\\` |\" >> $GITHUB_STEP_SUMMARY\n          echo \"| Number | \\`${{ inputs.number_input }}\\` |\" >> $GITHUB_STEP_SUMMARY\n          echo \"| Boolean | \\`${{ inputs.boolean_input }}\\` |\" >> $GITHUB_STEP_SUMMARY\n          echo \"| Choice | \\`${{ inputs.choice_input }}\\` |\" >> $GITHUB_STEP_SUMMARY\n          echo \"| Environment | \\`${{ inputs.environment_input }}\\` |\" >> $GITHUB_STEP_SUMMARY\n          echo \"\" >> $GITHUB_STEP_SUMMARY\n          echo \"---\" >> $GITHUB_STEP_SUMMARY\n          echo \"*Workflow completed successfully* ✨\" >> $GITHUB_STEP_SUMMARY\n\n"
            }
          }
        ]
      }
    }
  }
}
```


```rust
#[derive(Deserialize, Debug)]
pub struct GqlResponse {
    pub data: Data,
}

#[derive(Deserialize, Debug)]
pub struct Data {
    pub repository: Repository,
}

#[derive(Deserialize, Debug)]
pub struct Repository {
    pub refs: RefNodes,
    pub object: WorkflowTree,
}

#[derive(Deserialize, Debug)]
pub struct WorkflowTree {
    pub entries: Vec<WorkflowEntry>,
}

#[derive(Deserialize, Debug)]
pub struct WorkflowEntry {
    pub name: String,         // e.g., "ci.yml"
    pub object: BlobText,     // This contains the raw YAML
}

#[derive(Deserialize, Debug)]
pub struct BlobText {
    pub text: String,
}

// Separate struct for the REST metadata
#[derive(Deserialize, Debug)]
pub struct WorkflowMetadata {
    pub id: u64,
    pub state: String,        // "active", "deleted", etc.
    pub html_url: String,
    pub path: String,         // Use this to match against the GQL 'name'
}
```