mod common_tests;

use async_trait::async_trait;
use common_tests::fixtures::provider::acp_provider_factory;
use common_tests::fixtures::run_test;
use common_tests::fixtures::server::{AcpServerConnection, AcpServerSession};
use common_tests::fixtures::{Connection, OpenAiFixture, SessionData, TestConnectionConfig};
use common_tests::{
    run_close_session, run_config_mcp, run_config_option_mode_set, run_config_option_model_set,
    run_delete_session, run_fs_read_text_file_true, run_fs_write_text_file_false,
    run_fs_write_text_file_true, run_initialize_doesnt_hit_provider, run_list_sessions,
    run_load_mode, run_load_model, run_load_session_error, run_load_session_mcp, run_mode_set,
    run_model_list, run_model_set, run_model_set_error_session_not_found,
    run_permission_persistence, run_prompt_basic, run_prompt_codemode, run_prompt_error,
    run_prompt_image, run_prompt_image_attachment, run_prompt_mcp, run_prompt_model_mismatch,
    run_prompt_skill, run_shell_terminal_false, run_shell_terminal_true,
};
use goose::providers::base::Provider;
use goose_test_support::{ExpectedSessionId, IgnoreSessionId};
use sacp::schema::{AuthMethod, ListSessionsResponse, McpServer};
use std::sync::Arc;

struct ServerToAcpConnection(
    AcpServerConnection,
    #[allow(dead_code)] Option<tempfile::TempDir>,
    Arc<goose::config::PermissionManager>,
);

#[async_trait]
impl Connection for ServerToAcpConnection {
    type Session = AcpServerSession;

    // AcpProvider remaps session IDs
    fn expected_session_id() -> Arc<dyn ExpectedSessionId> {
        Arc::new(IgnoreSessionId)
    }

    async fn new(mut config: TestConnectionConfig, openai: OpenAiFixture) -> Self {
        let (data_root, _temp_dir) = match config.data_root.as_os_str().is_empty() {
            true => {
                let temp_dir = tempfile::tempdir().unwrap();
                (temp_dir.path().to_path_buf(), Some(temp_dir))
            }
            false => (config.data_root.clone(), None),
        };

        let cwd_path = config
            .cwd
            .as_ref()
            .map(|td| td.path().to_path_buf())
            .unwrap_or_else(|| data_root.clone());

        let notification_sink = Arc::new(std::sync::Mutex::new(Vec::new()));
        let mcp_servers = std::mem::take(&mut config.mcp_servers);

        let (provider, downstream_pm) = acp_provider_factory(
            &openai,
            &config.builtins,
            data_root.as_path(),
            config.goose_mode,
            config.provider_factory.take(),
            &config.current_model,
            &cwd_path,
            mcp_servers,
            config.strip_config_options,
            notification_sink,
        )
        .await;

        let acp_provider: Arc<dyn Provider> = Arc::new(provider);
        config.provider_factory = Some(Arc::new(move |_model_config, _extensions| {
            let p = acp_provider.clone();
            Box::pin(async move { Ok(p) })
        }));

        Self(
            AcpServerConnection::new(config, openai).await,
            _temp_dir,
            downstream_pm,
        )
    }

    async fn new_session(&mut self) -> anyhow::Result<SessionData<AcpServerSession>> {
        self.0.new_session().await
    }

    async fn load_session(
        &mut self,
        session_id: &str,
        mcp_servers: Vec<McpServer>,
    ) -> anyhow::Result<SessionData<AcpServerSession>> {
        self.0.load_session(session_id, mcp_servers).await
    }

    async fn list_sessions(&self) -> anyhow::Result<ListSessionsResponse> {
        self.0.list_sessions().await
    }

    async fn close_session(&self, session_id: &str) -> anyhow::Result<()> {
        self.0.close_session(session_id).await
    }

    async fn delete_session(&self, session_id: &str) -> anyhow::Result<()> {
        self.0.delete_session(session_id).await
    }

    async fn set_mode(&self, session_id: &str, mode_id: &str) -> anyhow::Result<()> {
        self.0.set_mode(session_id, mode_id).await
    }

    async fn set_model(&self, session_id: &str, model_id: &str) -> anyhow::Result<()> {
        self.0.set_model(session_id, model_id).await
    }

    async fn set_config_option(
        &self,
        session_id: &str,
        config_id: &str,
        value: &str,
    ) -> anyhow::Result<()> {
        self.0.set_config_option(session_id, config_id, value).await
    }

    fn auth_methods(&self) -> &[AuthMethod] {
        self.0.auth_methods()
    }

    fn data_root(&self) -> std::path::PathBuf {
        self.0.data_root()
    }

    fn reset_openai(&self) {
        self.0.reset_openai();
    }

    fn reset_permissions(&self) {
        self.0.reset_permissions();
        self.2.remove_extension("");
    }
}

tests_config_option_set_error!(ServerToAcpConnection);
tests_mode_set_error!(ServerToAcpConnection);

#[test]
fn test_config_mcp() {
    run_test(async { run_config_mcp::<ServerToAcpConnection>().await });
}

#[test]
fn test_config_option_mode_set() {
    run_test(async { run_config_option_mode_set::<ServerToAcpConnection>().await });
}

#[test]
fn test_list_sessions() {
    run_test(async { run_list_sessions::<ServerToAcpConnection>().await });
}

#[test]
fn test_close_session() {
    run_test(async { run_close_session::<ServerToAcpConnection>().await });
}

#[test]
fn test_config_option_model_set() {
    run_test(async { run_config_option_model_set::<ServerToAcpConnection>().await });
}

#[test]
fn test_delete_session() {
    run_test(async { run_delete_session::<ServerToAcpConnection>().await });
}

#[test]
#[ignore = "TODO: propagate client FS read capability to AcpProvider-backed session"]
fn test_fs_read_text_file_true() {
    run_test(async { run_fs_read_text_file_true::<ServerToAcpConnection>().await });
}

#[test]
fn test_fs_write_text_file_false() {
    run_test(async { run_fs_write_text_file_false::<ServerToAcpConnection>().await });
}

#[test]
#[ignore = "TODO: propagate client FS write capability to AcpProvider-backed session"]
fn test_fs_write_text_file_true() {
    run_test(async { run_fs_write_text_file_true::<ServerToAcpConnection>().await });
}

#[test]
fn test_initialize_doesnt_hit_provider() {
    run_test(async { run_initialize_doesnt_hit_provider::<ServerToAcpConnection>().await });
}

#[test]
fn test_load_mode() {
    run_test(async { run_load_mode::<ServerToAcpConnection>().await });
}

#[test]
fn test_load_model() {
    run_test(async { run_load_model::<ServerToAcpConnection>().await });
}

#[test]
fn test_load_session_error_session_not_found() {
    run_test(async { run_load_session_error::<ServerToAcpConnection>().await });
}

#[test]
fn test_load_session_mcp() {
    run_test(async { run_load_session_mcp::<ServerToAcpConnection>().await });
}

#[test]
fn test_mode_set() {
    run_test(async { run_mode_set::<ServerToAcpConnection>().await });
}

#[test]
fn test_model_list() {
    run_test(async { run_model_list::<ServerToAcpConnection>().await });
}

#[test]
fn test_model_set() {
    run_test(async { run_model_set::<ServerToAcpConnection>().await });
}

#[test]
fn test_model_set_error_session_not_found() {
    run_test(async { run_model_set_error_session_not_found::<ServerToAcpConnection>().await });
}

#[test]
fn test_permission_persistence() {
    run_test(async { run_permission_persistence::<ServerToAcpConnection>().await });
}

#[test]
fn test_prompt_basic() {
    run_test(async { run_prompt_basic::<ServerToAcpConnection>().await });
}

#[test]
fn test_prompt_codemode() {
    run_test(async { run_prompt_codemode::<ServerToAcpConnection>().await });
}

#[test]
fn test_prompt_error_session_not_found() {
    run_test(async { run_prompt_error::<ServerToAcpConnection>().await });
}

#[test]
fn test_prompt_image() {
    run_test(async { run_prompt_image::<ServerToAcpConnection>().await });
}

#[test]
fn test_prompt_image_attachment() {
    run_test(async { run_prompt_image_attachment::<ServerToAcpConnection>().await });
}

#[test]
fn test_prompt_mcp() {
    run_test(async { run_prompt_mcp::<ServerToAcpConnection>().await });
}

#[test]
fn test_prompt_model_mismatch() {
    run_test(async { run_prompt_model_mismatch::<ServerToAcpConnection>().await });
}

#[test]
fn test_prompt_skill() {
    run_test(async { run_prompt_skill::<ServerToAcpConnection>().await });
}

#[test]
fn test_shell_terminal_false() {
    run_test(async { run_shell_terminal_false::<ServerToAcpConnection>().await });
}

#[test]
#[ignore = "TODO: propagate client terminal capability to AcpProvider-backed session"]
fn test_shell_terminal_true() {
    run_test(async { run_shell_terminal_true::<ServerToAcpConnection>().await });
}
