use std::sync::Arc;

use crate::{AgentTool, ContextServerRegistry, Templates, Thread, ToolCallEventStream};
use agent_client_protocol as acp;
use anyhow::Result;
use gpui::{App, Entity, SharedString, Task};
use project::Project;
use prompt_store::ProjectContext;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Create a new assistant thread associated with the current project.
///
/// Use this when you want to:
/// - Start a fresh conversation separate from the current thread
/// - Organize work into sub-threads (for example, per-task or per-file)
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateThreadToolInput {
    /// Optional title for the new thread.
    ///
    /// If omitted, a default title will be used.
    pub title: Option<String>,
}

/// Tool that creates a new [`Thread`](crate::Thread) for the current project.
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

    fn kind() -> acp::ToolKind {
        acp::ToolKind::Other
    }

    fn initial_title(
        &self,
        input: Result<Self::Input, serde_json::Value>,
        _cx: &mut App,
    ) -> SharedString {
        if let Ok(CreateThreadToolInput {
            title: Some(title),
        }) = input
        {
            format!("Create thread: {title}").into()
        } else {
            "Create thread".into()
        }
    }

    fn run(
        self: Arc<Self>,
        input: Self::Input,
        _event_stream: ToolCallEventStream,
        cx: &mut App,
    ) -> Task<Result<Self::Output>> {
        let title = input.title;
        cx.spawn({
            let this = Arc::clone(&self);
            async move |cx| {
                let thread = cx.new(|cx| {
                    Thread::new(
                        this.project.clone(),
                        this.project_context.clone(),
                        this.context_server_registry.clone(),
                        this.templates.clone(),
                        None,
                        cx,
                    )
                });

                if let Some(title) = title {
                    thread.update(cx, |thread, cx| {
                        thread.set_title(title.into(), cx);
                        Ok(())
                    })?;
                }

                Ok(thread.entity_id().to_string())
            }
        })
    }
}