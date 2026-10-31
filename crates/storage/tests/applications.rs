use open_omaha_domain::{Application, CupPolicy, VersionScheme};
use open_omaha_storage::Store;

fn application(id: &str, omaha_app_id: &str) -> Application {
    Application {
        id: id.into(),
        omaha_app_id: omaha_app_id.into(),
        display_name: "Example application".into(),
        version_scheme: VersionScheme::default(),
        cup_policy: CupPolicy::default(),
    }
}

async fn memory_store() -> Store {
    let store = Store::connect("sqlite::memory:").await.unwrap();
    store.migrate().await.unwrap();
    store
}

#[tokio::test]
async fn migrations_are_repeatable_and_missing_applications_return_none() {
    let store = memory_store().await;
    store.migrate().await.unwrap();
    assert_eq!(store.get_application("missing").await.unwrap(), None);
}

#[tokio::test]
async fn application_survives_database_reopen() {
    let directory = tempfile::tempdir().unwrap();
    let url = format!(
        "sqlite://{}",
        directory.path().join("applications.db").display()
    );
    let app = application("example", "Com.Example.App");
    {
        let store = Store::connect(&url).await.unwrap();
        store.migrate().await.unwrap();
        store.create_application(&app).await.unwrap();
    }
    let store = Store::connect(&url).await.unwrap();
    assert_eq!(store.get_application("example").await.unwrap(), Some(app));
}

#[tokio::test]
async fn management_ids_and_ascii_case_insensitive_omaha_ids_are_unique() {
    let store = memory_store().await;
    let original = application("example", "Com.Example.App");
    store.create_application(&original).await.unwrap();
    for duplicate in [
        application("example", "different-app"),
        application("other", "com.example.app"),
    ] {
        let error = store.create_application(&duplicate).await.unwrap_err();
        assert!(error.as_database_error().unwrap().is_unique_violation());
    }
    assert_eq!(
        store.get_application("example").await.unwrap(),
        Some(original)
    );
    assert_eq!(store.get_application("other").await.unwrap(), None);
}

#[tokio::test]
async fn version_schemes_and_cup_policies_round_trip() {
    let store = memory_store().await;
    for (id, scheme, policy) in [
        ("chromium", VersionScheme::Chromium, CupPolicy::Required),
        ("semver", VersionScheme::Semver, CupPolicy::Optional),
        ("legacy", VersionScheme::OmahaLegacy, CupPolicy::Required),
    ] {
        let mut app = application(id, id);
        app.version_scheme = scheme;
        app.cup_policy = policy;
        store.create_application(&app).await.unwrap();
        assert_eq!(store.get_application(id).await.unwrap(), Some(app));
    }
}
