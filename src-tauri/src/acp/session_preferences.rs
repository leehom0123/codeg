use sea_orm::DatabaseConnection;

use crate::db::service::app_metadata_service;
use crate::models::AgentType;

use super::connection::current_config_option_values;
use super::delegation::types::AgentDelegationDefaults;
use super::error::AcpError;
use super::types::SessionConfigOptionInfo;

pub(crate) fn from_current(
    mode_id: Option<String>,
    options: &[SessionConfigOptionInfo],
) -> AgentDelegationDefaults {
    let mut values = current_config_option_values(options);
    values.retain(|id, _| {
        matches!(
            id.as_str(),
            "model" | "mode" | "reasoning_effort" | "effort" | "thought_level"
        ) || options.iter().any(|option| {
            option.id == *id
                && matches!(
                    option.category.as_deref(),
                    Some("model" | "mode" | "thought_level")
                )
        })
    });
    // When a picker exposes the permission as a config option it owns replay.
    let mode_id = mode_id.filter(|_| {
        !options.iter().any(|option| {
            option.category.as_deref() == Some("mode") && values.contains_key(&option.id)
        })
    });
    AgentDelegationDefaults {
        mode_id,
        config_values: values,
    }
}

fn key(agent: AgentType, session_id: &str) -> String {
    format!("acp-session-preferences:{agent}:{session_id}")
}

pub(crate) async fn load(
    db: &DatabaseConnection,
    agent: AgentType,
    session_id: Option<&str>,
) -> Result<Option<AgentDelegationDefaults>, AcpError> {
    let Some(session_id) = session_id else {
        return Ok(None);
    };
    app_metadata_service::get_value(db, &key(agent, session_id))
        .await
        .map_err(|error| AcpError::protocol(error.to_string()))?
        .map(|value| {
            serde_json::from_str(&value).map_err(|error| AcpError::protocol(error.to_string()))
        })
        .transpose()
}

pub(crate) async fn save(
    db: &DatabaseConnection,
    agent: AgentType,
    session_id: &str,
    preferences: &AgentDelegationDefaults,
) -> Result<(), AcpError> {
    let value = serde_json::to_string(preferences)
        .map_err(|error| AcpError::protocol(error.to_string()))?;
    app_metadata_service::upsert_value(db, &key(agent, session_id), &value)
        .await
        .map_err(|error| AcpError::protocol(error.to_string()))
}

#[cfg(test)]
#[tokio::test]
async fn web_selector_change_is_saved_for_resume() {
    use std::collections::BTreeMap;
    use std::sync::Arc;

    use axum_test::TestServer;
    use serde_json::json;

    use crate::acp::connection::{ConnectionCommand, SetterOutcome};
    use crate::app_state::AppState;
    use crate::db::test_helpers::fresh_in_memory_db;
    use crate::web::{router::build_router, shutdown::ShutdownSignal};

    let db = fresh_in_memory_db().await;
    let data_dir = std::env::current_dir().unwrap();
    let conn = db.conn.clone();
    let app = Arc::new(AppState::new_for_test(db, data_dir.clone()));
    let mut commands = app
        .connection_manager
        .insert_test_connection_live("recipient", AgentType::Codex, None, app.emitter.clone())
        .await;
    let state = app.connection_manager.get_state("recipient").await.unwrap();
    state.write().await.external_id = Some("recipient-session".into());
    let agent = tokio::spawn(async move {
        let Some(ConnectionCommand::SetConfigOption {
            config_id,
            value_id,
            reply: Some(reply),
        }) = commands.recv().await
        else {
            panic!("expected an acknowledged config setter");
        };
        assert_eq!(
            (config_id.as_str(), value_id.as_str()),
            ("reasoning_effort", "medium")
        );
        state.write().await.config_options = Some(serde_json::from_value(json!([
            { "id": "model", "name": "Model", "category": "model",
              "kind": { "type": "select", "current_value": "gpt-6.2-sol", "options": [], "groups": [] } },
            { "id": "mode", "name": "Mode", "category": "mode",
              "kind": { "type": "select", "current_value": "agent-full-access", "options": [], "groups": [] } },
            { "id": "reasoning_effort", "name": "Effort", "category": "thought_level",
              "kind": { "type": "select", "current_value": "medium", "options": [], "groups": [] } }
        ])).unwrap());
        reply
            .send(Ok(SetterOutcome::Confirmed { current: value_id }))
            .unwrap();
    });
    let server = TestServer::builder()
        .http_transport()
        .build(build_router(
            app,
            "test-token".into(),
            data_dir,
            Arc::new(ShutdownSignal::new()),
        ))
        .unwrap();
    let response = server.post("/api/acp_set_config_option")
        .add_header("Authorization", "Bearer test-token")
        .json(&json!({ "connectionId": "recipient", "configId": "reasoning_effort", "valueId": "medium" }))
        .await;
    response.assert_status_ok();
    agent.await.unwrap();

    let saved = load(&conn, AgentType::Codex, Some("recipient-session"))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        saved.config_values,
        BTreeMap::from([
            ("model".into(), "gpt-6.2-sol".into()),
            ("mode".into(), "agent-full-access".into()),
            ("reasoning_effort".into(), "medium".into()),
        ])
    );
    assert!(load(&conn, AgentType::Codex, Some("other-session"))
        .await
        .unwrap()
        .is_none());
}
