use chrono::{TimeDelta, Utc};
use oneclient_auth::{AccountKind, AuthError, CredentialsStore, offline_account, offline_uuid};
use uuid::Uuid;

fn isolate_launcher_dir() {
    oneclient_common::paths::set_launcher_dir(
        std::env::temp_dir().join(format!("oneclient-auth-test-{}", Uuid::new_v4())),
    );
}

/// The launcher directory can only be set once per process, so every test here
/// shares one `auth.json`. Tests that write or reload it take turns instead of
/// racing each other over the file.
fn disk_lock() -> &'static tokio::sync::Mutex<()> {
    static LOCK: std::sync::OnceLock<tokio::sync::Mutex<()>> = std::sync::OnceLock::new();
    LOCK.get_or_init(|| tokio::sync::Mutex::new(()))
}

fn fake_microsoft_account(username: &str) -> oneclient_auth::MinecraftAccount {
    let mut account = offline_account(username.to_string());
    account.kind = AccountKind::Microsoft;
    account.access_token = "access".into();
    account.refresh_token = "refresh".into();
    account
}

#[test]
fn offline_uuid_matches_vanilla_algorithm() {
    let id = offline_uuid("Notch");

    assert_eq!(
        id,
        Uuid::parse_str("b50ad385-829d-3141-a216-7e7d7539ba7f").unwrap()
    );
    assert_ne!(id, offline_uuid("Steve"));
}

#[tokio::test]
async fn offline_account_allowed_without_microsoft() {
    isolate_launcher_dir();

    let mut store = CredentialsStore::default();
    let offline = store.add_offline_account("Steve".into()).unwrap();

    assert_eq!(offline.kind, AccountKind::Offline);
    assert_eq!(offline.username, "Steve");
    assert_eq!(
        store.default_user,
        Some(offline.id),
        "the first account is a usable default whether or not it is Microsoft"
    );
}

#[tokio::test]
async fn default_offline_still_works_when_last_microsoft_removed() {
    isolate_launcher_dir();

    let mut store = CredentialsStore::default();
    let msa = fake_microsoft_account("MsaUser");
    let msa_id = msa.id;
    store.users.insert(msa_id, msa);

    let offline = store.add_offline_account("Steve".into()).unwrap();
    store.default_user = Some(offline.id);
    store.users.remove(&msa_id);

    let account = store
        .default_account()
        .await
        .expect("removing Microsoft must not break offline play")
        .expect("the offline account is still the default");

    assert_eq!(account.id, offline.id);
    assert!(account.is_offline());
}

#[test]
fn duplicate_offline_names_collide_exactly_but_case_differs() {
    isolate_launcher_dir();

    let mut store = CredentialsStore::default();
    let first = store.add_offline_account("Steve".into()).unwrap();

    let err = store.add_offline_account("Steve".into()).unwrap_err();
    assert!(matches!(err, AuthError::DuplicateUsername { .. }));

    // Vanilla derives the UUID from the exact name so case variants are
    // distinct accounts, not a duplicate
    let upper = store.add_offline_account("STEVE".into()).unwrap();
    assert_ne!(upper.id, first.id);
    assert_eq!(store.default_user, Some(first.id));
}

#[tokio::test]
async fn renaming_an_offline_account_recomputes_its_uuid_and_follows_the_default() {
    isolate_launcher_dir();
    let _guard = disk_lock().lock().await;

    let mut store = CredentialsStore::default();
    let original = store.add_offline_account("Steve".into()).unwrap();

    let renamed = store
        .rename_offline_account(original.id, "Alex".into())
        .await
        .unwrap();

    assert_eq!(renamed.username, "Alex");
    assert_eq!(renamed.id, offline_uuid("Alex"));
    assert!(store.get_account(original.id).is_none());
    assert_eq!(store.default_user, Some(renamed.id));

    let reloaded = CredentialsStore::load().await.unwrap();
    assert_eq!(reloaded.default_user, Some(renamed.id));
    assert!(reloaded.get_account(renamed.id).is_some());
}

#[tokio::test]
async fn renaming_onto_a_taken_name_is_refused() {
    isolate_launcher_dir();

    let mut store = CredentialsStore::default();
    store.add_offline_account("Steve".into()).unwrap();
    let alex = store.add_offline_account("Alex".into()).unwrap();

    let err = store
        .rename_offline_account(alex.id, "Steve".into())
        .await
        .unwrap_err();

    assert!(matches!(err, AuthError::DuplicateUsername { .. }));
    assert!(store.get_account(alex.id).is_some());
}

#[tokio::test]
async fn a_microsoft_account_cannot_be_renamed_as_offline() {
    isolate_launcher_dir();

    let mut store = CredentialsStore::default();
    let msa = fake_microsoft_account("MsaUser");
    let msa_id = msa.id;
    store.users.insert(msa_id, msa);

    let err = store
        .rename_offline_account(msa_id, "Other".into())
        .await
        .unwrap_err();

    assert!(matches!(err, AuthError::NotOfflineAccount(_)));
    assert_eq!(store.get_account(msa_id).unwrap().username, "MsaUser");
}

#[tokio::test]
async fn removing_the_only_offline_account_is_allowed() {
    isolate_launcher_dir();
    let _guard = disk_lock().lock().await;

    let mut store = CredentialsStore::default();
    let offline = store.add_offline_account("Steve".into()).unwrap();

    let removed = store.remove_account(offline.id).await.unwrap();

    assert!(removed.is_some());
    assert_eq!(store.default_user, None);
    assert!(store.default_account().await.unwrap().is_none());
}

#[tokio::test]
async fn adding_microsoft_later_keeps_an_offline_default() {
    isolate_launcher_dir();
    let _guard = disk_lock().lock().await;

    let (events, _rx) = oneclient_events::EventBus::channel();
    let mut store = CredentialsStore::default();
    let offline = store.add_offline_account("Steve".into()).unwrap();

    store
        .commit_account(fake_microsoft_account("MsaUser"), &events)
        .await
        .unwrap();

    assert_eq!(
        store.default_user,
        Some(offline.id),
        "signing in later must not silently switch the active account"
    );
}

#[test]
fn token_expiring_imminently_counts_as_expired() {
    let mut account = fake_microsoft_account("MsaUser");

    account.expires = Utc::now() + TimeDelta::hours(1);
    assert!(!account.is_expired(), "a token with an hour left is usable");

    account.expires = Utc::now() + TimeDelta::seconds(5);
    assert!(
        account.is_expired(),
        "a token about to lapse must be renewed before launching"
    );

    account.expires = Utc::now() - TimeDelta::hours(1);
    assert!(account.is_expired());
}

#[tokio::test]
async fn default_account_returns_expired_token_without_refreshing() {
    isolate_launcher_dir();

    let mut store = CredentialsStore::default();
    let mut msa = fake_microsoft_account("MsaUser");
    msa.expires = Utc::now() - TimeDelta::hours(5);
    let msa_id = msa.id;
    store.users.insert(msa_id, msa);

    store.default_user = Some(msa_id);

    let account = store
        .default_account()
        .await
        .expect("an expired token must not fail the read path")
        .expect("the only account should be the default");

    assert_eq!(account.id, msa_id);
    assert!(account.is_expired());
}

#[tokio::test]
async fn signed_out_account_stays_listed_and_default() {
    isolate_launcher_dir();
    let _guard = disk_lock().lock().await;

    let mut store = CredentialsStore::default();
    let msa = fake_microsoft_account("MsaUser");
    let msa_id = msa.id;
    store.users.insert(msa_id, msa);
    store.default_user = Some(msa_id);

    store.mark_signed_out(msa_id).await.unwrap();

    let reloaded = CredentialsStore::load().await.unwrap();
    let account = reloaded.get_account(msa_id).expect("entry must survive");
    assert!(account.needs_sign_in());
    assert!(
        !account.refresh_token.is_empty(),
        "the refresh token is kept so the next renewal can retry it"
    );
    assert!(account.access_token.is_empty());
    assert_eq!(reloaded.default_user, Some(msa_id));
}

#[test]
fn accounts_saved_before_the_flag_existed_load_as_signed_in() {
    let account: oneclient_auth::MinecraftAccount = serde_json::from_str(&format!(
        r#"{{"id":"{}","username":"MsaUser","access_token":"a","refresh_token":"r","expires":"2026-01-01T00:00:00Z","kind":"microsoft"}}"#,
        Uuid::new_v4()
    ))
    .unwrap();

    assert!(!account.needs_sign_in());
}

#[test]
fn offline_accounts_never_need_sign_in() {
    let mut account = offline_account("Steve".into());
    account.signed_out = true;

    assert!(!account.needs_sign_in());
}
