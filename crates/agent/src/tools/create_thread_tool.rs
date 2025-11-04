use crate::{AgentTool, ContextServerRegistry, Templates, Thread, ToolCallEventStream};
use agent_client_protocol::ToolKind;
use anyhow::{Context as _, Result};
use gpui::{App, Entity, Task};
use project::Project;
use prompt_store::ProjectContext;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Creates a new agent thread with the current project context.
///
/// This tool allows agents to create new threads programmatically,
/// enabling them to organize their work or spawn sub-tasks.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateThreadToolInput {
    /// Optional title for the new thread.
    ///
    /// <example>
    /// To create a thread titled "Code Review", provide a title of "Code Review"
    /// </example>
    pub title: Option<String>,
}

pub struct CreateThreadTool {
    project: Entity<Project>,
    project_context: Entity<ProjectContext>,
    context_server_registry: Entity<ContextServerRegistry>,
    templates: Arc<Templates>,
}

impl CreateThreadTool {
    pub fn new(
        project: Entity<Project>,
        project_context: Entity<ProjectContext>,
        context_server_registry: Entity<ContextServerRegistry>,
        templates: Arc<Templates>,
    ) -> Self {
        Self {
            project,
            project_context,
            context_server_registry,
            templates,
        }
    }
}

impl AgentTool for CreateThreadTool {
    type Input = CreateThreadToolInput;
    type Output = String;

    fn name() -> &'static str {
        "create_thread"
    }

    fn kind() -> ToolKind {
        ToolKind::Other
    }

    fn initial_title(
        &self,
        input: Result<Self::Input, serde_json::Value>,
        _cx: &mut App,
    ) -> ui::SharedString {
        if let Ok(input) = input {
            if let Some(title) = input.title {
                return format!("Create Thread: {}", title).into();
            }
        }
        "Create Thread".into()
    }

    fn run(
        self: Arc<Self>,
        input: Self::Input,
        _event_stream: ToolCallEventStream,
        cx: &mut App,
    ) -> Task<Result<Self::Output>> {
        let thread_task = cx.spawn(|mut cx: &mut App| async move {
            let thread = cx.new(|cx| {
                Thread::new(
                    self.project.clone(),
                    self.project_context.clone(),
                    self.context_server_registry.clone(),
                    self.templates.clone(),
                    None, // No model specified, will use default
                    cx,
                )
            })?;
            
            // Set the title if provided
            if let Some(title) = input.title {
                thread.update(&mut cx, |thread, cx| {
                    thread.set_title(title.into(), cx);
                })?;
            }
            
            Ok(thread.entity_id().to_string())
        });

        thread_task
    }
}