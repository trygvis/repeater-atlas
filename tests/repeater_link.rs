mod utils;

use diesel_async::AsyncConnection;
use repeater_atlas::RepeaterAtlasError;
use repeater_atlas::dao;
use repeater_atlas::dao::repeater_link::NewRepeaterLink;
use repeater_atlas::dao::repeater_system::NewRepeaterSystem;
use repeater_atlas::service;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tokio::sync::Mutex;

#[derive(Debug, Error)]
enum TestError {
    #[error("database error")]
    Database(#[from] diesel::result::Error),
    #[error("{0}")]
    Other(String),
    #[error("rollback")]
    Rollback,
}

impl From<RepeaterAtlasError> for TestError {
    fn from(error: RepeaterAtlasError) -> Self {
        match error {
            RepeaterAtlasError::Database(error) => Self::Database(error),
            RepeaterAtlasError::DatabaseOther(error, _) => Self::Database(error),
            other => Self::Other(format!("{other}")),
        }
    }
}

static TEST_LOCK: Mutex<()> = Mutex::const_new(());

#[derive(Clone)]
struct ExpectedPath {
    call_sign_key: &'static str,
    depth: i32,
    path_keys: Vec<&'static str>,
}

impl ExpectedPath {
    fn new(call_sign_key: &'static str, depth: i32, path_keys: Vec<&'static str>) -> Self {
        Self {
            call_sign_key,
            depth,
            path_keys,
        }
    }
}

#[tokio::test]
async fn finds_linked_repeaters_paths_branched() -> Result<(), TestError> {
    run_case(
        "branched_links",
        vec!["A", "B", "C", "D"],
        vec![("B", "A"), ("B", "C"), ("B", "D")],
        "A",
        vec![
            ExpectedPath::new("A", 0, vec!["A"]),
            ExpectedPath::new("B", 1, vec!["A", "B"]),
            ExpectedPath::new("C", 2, vec!["A", "B", "C"]),
            ExpectedPath::new("D", 2, vec!["A", "B", "D"]),
        ],
        1,
    )
    .await
}

#[tokio::test]
async fn finds_linked_repeaters_paths_isolated() -> Result<(), TestError> {
    run_case(
        "isolated_start",
        vec!["A"],
        vec![],
        "A",
        vec![ExpectedPath::new("A", 0, vec!["A"])],
        2,
    )
    .await
}

#[tokio::test]
async fn finds_linked_repeaters_paths_linear() -> Result<(), TestError> {
    run_case(
        "linear_chain",
        vec!["C", "A", "B"],
        // Intentionally mix input order to ensure the service layer handles it.
        vec![("B", "A"), ("C", "B")],
        "A",
        vec![
            ExpectedPath::new("A", 0, vec!["A"]),
            ExpectedPath::new("B", 1, vec!["A", "B"]),
            ExpectedPath::new("C", 2, vec!["A", "B", "C"]),
        ],
        3,
    )
    .await
}

async fn run_case(
    name: &'static str,
    call_sign_keys: Vec<&'static str>,
    links: Vec<(&'static str, &'static str)>,
    start_key: &'static str,
    expected: Vec<ExpectedPath>,
    case_tag: u32,
) -> Result<(), TestError> {
    let _guard = TEST_LOCK.lock().await;
    let pool = utils::pool().await;
    let mut c = pool
        .get()
        .await
        .map_err(|e| TestError::Other(format!("{e}")))?;

    let result = c
        .transaction(move |c| {
            Box::pin(async move {
                let base = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|e| TestError::Other(format!("time error: {e}")))?
                    .as_nanos()
                    % 1_000_000;

                let call_signs: HashMap<&str, String> = call_sign_keys
                    .iter()
                    .map(|key| (*key, format!("T{:06}{case_tag}{key}", base)))
                    .collect();

                let mut repeater_ids = HashMap::new();
                for call_sign in call_signs.values() {
                    let repeater = service::repeater_system::create_repeater_system(
                        c,
                        NewRepeaterSystem::new(call_sign.clone()),
                    )
                    .await
                    .map_err(TestError::from)?;
                    repeater_ids.insert(call_sign.as_str(), repeater.id);
                }

                for (left_key, right_key) in links.iter() {
                    let left = call_signs.get(left_key).ok_or_else(|| {
                        TestError::Other(format!("missing call sign for {left_key}"))
                    })?;
                    let right = call_signs.get(right_key).ok_or_else(|| {
                        TestError::Other(format!("missing call sign for {right_key}"))
                    })?;
                    let left_id = *repeater_ids.get(left.as_str()).ok_or_else(|| {
                        TestError::Other(format!("missing repeater id for {left}"))
                    })?;
                    let right_id = *repeater_ids.get(right.as_str()).ok_or_else(|| {
                        TestError::Other(format!("missing repeater id for {right}"))
                    })?;
                    dao::repeater_link::insert(c, NewRepeaterLink::new(left_id, right_id))
                        .await
                        .map_err(TestError::from)?;
                }

                let start_call_sign = call_signs.get(start_key).ok_or_else(|| {
                    TestError::Other(format!("missing call sign for {start_key}"))
                })?;
                let linked = dao::repeater_link::find_linked_repeaters(c, start_call_sign.clone())
                    .await
                    .map_err(TestError::from)?;
                let result: Vec<(String, i32, Vec<String>)> = linked
                    .into_iter()
                    .map(|row| (row.call_sign, row.depth, row.path))
                    .collect();

                let expected: Vec<(String, i32, Vec<String>)> = expected
                    .iter()
                    .map(|row| {
                        let call_sign = call_signs
                            .get(row.call_sign_key)
                            .ok_or_else(|| {
                                TestError::Other(format!(
                                    "missing call sign for {}",
                                    row.call_sign_key
                                ))
                            })?
                            .clone();
                        let path = row
                            .path_keys
                            .iter()
                            .map(|key| {
                                call_signs.get(*key).cloned().ok_or_else(|| {
                                    TestError::Other(format!("missing call sign for {key}"))
                                })
                            })
                            .collect::<Result<Vec<String>, TestError>>()?;
                        Ok((call_sign, row.depth, path))
                    })
                    .collect::<Result<_, TestError>>()?;

                assert_eq!(result, expected, "case {}", name);

                Err(TestError::Rollback)
            })
        })
        .await;

    match result {
        Err(TestError::Rollback) => Ok(()),
        other => other,
    }
}

#[tokio::test]
async fn rejects_self_link_insert() -> Result<(), TestError> {
    let _guard = TEST_LOCK.lock().await;
    let pool = utils::pool().await;
    let mut c = pool
        .get()
        .await
        .map_err(|e| TestError::Other(format!("{e}")))?;

    let result = c
        .transaction(|c| {
            Box::pin(async move {
                let base = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|e| TestError::Other(format!("time error: {e}")))?
                    .as_nanos()
                    % 1_000_000;
                let call_sign = format!("T{:06}S", base);
                let repeater = service::repeater_system::create_repeater_system(
                    c,
                    NewRepeaterSystem::new(call_sign),
                )
                .await
                .map_err(TestError::from)?;

                let self_link = c
                    .transaction(move |c| {
                        Box::pin(async move {
                            dao::repeater_link::insert(
                                c,
                                NewRepeaterLink::new(repeater.id, repeater.id),
                            )
                            .await
                            .map(|_| ())
                        })
                    })
                    .await
                    .expect_err("expected self-link to fail");
                assert!(
                    matches!(self_link, diesel::result::Error::DatabaseError(_, _)),
                    "expected database constraint error for self-link"
                );

                Err(TestError::Rollback)
            })
        })
        .await;

    match result {
        Err(TestError::Rollback) => Ok(()),
        other => other,
    }
}
