use goose::providers::api_client::{ApiClient, AuthMethod as ApiAuthMethod};
use goose::providers::base::Provider;
use goose::providers::openai::OpenAiProvider;
use goose::providers::provider_registry::ProviderConstructor;
use std::sync::Arc;

#[allow(dead_code)]
pub fn llm_provider_factory(openai_base_url: &str) -> ProviderConstructor {
    let base_url = openai_base_url.to_string();
    Arc::new(move |model_config, _extensions| {
        let base_url = base_url.clone();
        Box::pin(async move {
            let api_client =
                ApiClient::new(base_url, ApiAuthMethod::BearerToken("test-key".into())).unwrap();
            let provider: Arc<dyn Provider> =
                Arc::new(OpenAiProvider::new(api_client, model_config));
            Ok(provider)
        })
    })
}
