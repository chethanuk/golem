// Mock API client for testing
use golem_client::model::{Component, ComponentView, Worker, WorkerStatus, Profile};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use wiremock::{MockServer, Mock, ResponseTemplate, matchers::{method, path}};

pub struct MockApiSetup {
    pub server: MockServer,
    pub component_id: String,
}

impl MockApiSetup {
    pub async fn new() -> Self {
        let server = MockServer::start().await;
        let component_id = "f4c4652b-9818-4178-b7c4-d486fa2aefae".to_string();
        
        Self {
            server,
            component_id,
        }
    }
    
    pub async fn mock_list_components(&self, components: Vec<ComponentView>) {
        Mock::given(method("GET"))
            .and(path("/v1/components"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(&components))
            .mount(&self.server)
            .await;
    }
    
    pub async fn mock_deploy_component(&self, component: ComponentView) {
        Mock::given(method("POST"))
            .and(path("/v1/components"))
            .respond_with(ResponseTemplate::new(201)
                .set_body_json(&component))
            .mount(&self.server)
            .await;
    }
    
    pub async fn mock_list_workers(&self, workers: Vec<Worker>) {
        Mock::given(method("GET"))
            .and(path(format!("/v1/components/{}/workers", self.component_id)))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(&workers))
            .mount(&self.server)
            .await;
    }
    
    pub async fn mock_create_worker(&self, worker: Worker) {
        Mock::given(method("POST"))
            .and(path(format!("/v1/components/{}/workers", self.component_id)))
            .respond_with(ResponseTemplate::new(201)
                .set_body_json(&worker))
            .mount(&self.server)
            .await;
    }
    
    pub async fn mock_invoke_worker(&self, result: serde_json::Value) {
        Mock::given(method("POST"))
            .and(path(format!("/v1/components/{}/workers/test-1/invoke", self.component_id)))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(&result))
            .mount(&self.server)
            .await;
    }
    
    pub fn api_url(&self) -> String {
        self.server.uri()
    }
}

pub fn create_test_component() -> ComponentView {
    ComponentView {
        component_id: Uuid::parse_str("f4c4652b-9818-4178-b7c4-d486fa2aefae").unwrap(),
        component_name: "shopping-cart".to_string(),
        component_version: 0,
        project_id: Some(Uuid::parse_str("c4118348-8bf2-4a4f-80bb-d99cc4aceb6b").unwrap()),
        component_type: "Durable".to_string(),
        component_size: 2_950_000,
        created_at: chrono::Utc::now(),
        exports: vec![
            "initialize-cart".to_string(),
            "add-item".to_string(),
            "remove-item".to_string(),
            "get-cart-contents".to_string(),
            "checkout".to_string(),
        ],
    }
}

pub fn create_test_worker() -> Worker {
    Worker {
        worker_id: WorkerId {
            component_id: Uuid::parse_str("f4c4652b-9818-4178-b7c4-d486fa2aefae").unwrap(),
            worker_name: "test-1".to_string(),
        },
        status: WorkerStatus::Running,
        created_at: chrono::Utc::now(),
        last_update: chrono::Utc::now(),
        metadata: Default::default(),
    }
}