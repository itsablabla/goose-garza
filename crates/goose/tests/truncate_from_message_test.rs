use goose::conversation::message::Message;
use goose::session::session_manager::SessionStorage;
use goose::session::thread_manager::ThreadManager;
use std::sync::Arc;
use tempfile::TempDir;

async fn setup() -> (ThreadManager, TempDir) {
    let tmp = TempDir::new().unwrap();
    let storage = SessionStorage::create(tmp.path()).await.unwrap();
    let tm = ThreadManager::new(Arc::new(storage));
    (tm, tmp)
}

/// Helper: append a user message with a known ID and return that ID.
async fn append_user(tm: &ThreadManager, thread_id: &str, text: &str) -> String {
    let mut msg = Message::user().with_text(text);
    let id = format!("test_{}", uuid::Uuid::new_v4());
    msg.id = Some(id.clone());
    tm.append_message(thread_id, Some("s1"), &msg)
        .await
        .unwrap();
    id
}

/// Helper: append an assistant message and return its ID.
async fn append_assistant(tm: &ThreadManager, thread_id: &str, text: &str) -> String {
    let mut msg = Message::assistant().with_text(text);
    let id = format!("test_{}", uuid::Uuid::new_v4());
    msg.id = Some(id.clone());
    tm.append_message(thread_id, Some("s1"), &msg)
        .await
        .unwrap();
    id
}

#[tokio::test]
async fn truncate_removes_target_and_subsequent_messages() {
    let (tm, _tmp) = setup().await;
    let thread = tm.create_thread(None, None, None).await.unwrap();

    let id1 = append_user(&tm, &thread.id, "msg 1").await;
    let _id2 = append_assistant(&tm, &thread.id, "reply 1").await;
    let id3 = append_user(&tm, &thread.id, "msg 2").await;
    let _id4 = append_assistant(&tm, &thread.id, "reply 2").await;

    // Truncate from msg 2 onward
    let (deleted, ts) = tm.truncate_from_message(&thread.id, &id3).await.unwrap();
    assert_eq!(deleted, 2, "should delete msg 2 + reply 2");
    assert!(ts > 0, "should return a valid timestamp");

    let remaining = tm.list_messages(&thread.id).await.unwrap();
    assert_eq!(remaining.len(), 2);
    assert_eq!(remaining[0].id.as_deref(), Some(id1.as_str()));
}

#[tokio::test]
async fn truncate_from_first_message_clears_all() {
    let (tm, _tmp) = setup().await;
    let thread = tm.create_thread(None, None, None).await.unwrap();

    let id1 = append_user(&tm, &thread.id, "first").await;
    let _id2 = append_assistant(&tm, &thread.id, "reply").await;

    let (deleted, _ts) = tm.truncate_from_message(&thread.id, &id1).await.unwrap();
    assert_eq!(deleted, 2);

    let remaining = tm.list_messages(&thread.id).await.unwrap();
    assert!(remaining.is_empty());
}

#[tokio::test]
async fn truncate_nonexistent_message_is_noop() {
    let (tm, _tmp) = setup().await;
    let thread = tm.create_thread(None, None, None).await.unwrap();

    append_user(&tm, &thread.id, "keep me").await;

    let (deleted, ts) = tm
        .truncate_from_message(&thread.id, "does_not_exist")
        .await
        .unwrap();
    assert_eq!(deleted, 0);
    assert_eq!(ts, 0);

    let remaining = tm.list_messages(&thread.id).await.unwrap();
    assert_eq!(remaining.len(), 1);
}

#[tokio::test]
async fn truncate_last_message_only() {
    let (tm, _tmp) = setup().await;
    let thread = tm.create_thread(None, None, None).await.unwrap();

    let _id1 = append_user(&tm, &thread.id, "keep").await;
    let _id2 = append_assistant(&tm, &thread.id, "keep too").await;
    let id3 = append_user(&tm, &thread.id, "delete me").await;

    let (deleted, _ts) = tm.truncate_from_message(&thread.id, &id3).await.unwrap();
    assert_eq!(deleted, 1);

    let remaining = tm.list_messages(&thread.id).await.unwrap();
    assert_eq!(remaining.len(), 2);
}

#[tokio::test]
async fn truncate_does_not_affect_other_threads() {
    let (tm, _tmp) = setup().await;
    let thread_a = tm.create_thread(None, None, None).await.unwrap();
    let thread_b = tm.create_thread(None, None, None).await.unwrap();

    let id_a1 = append_user(&tm, &thread_a.id, "thread A msg").await;
    append_user(&tm, &thread_b.id, "thread B msg").await;

    tm.truncate_from_message(&thread_a.id, &id_a1)
        .await
        .unwrap();

    let remaining_a = tm.list_messages(&thread_a.id).await.unwrap();
    let remaining_b = tm.list_messages(&thread_b.id).await.unwrap();
    assert!(remaining_a.is_empty());
    assert_eq!(remaining_b.len(), 1, "thread B should be untouched");
}
