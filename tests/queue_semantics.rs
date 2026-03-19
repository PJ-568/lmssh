use std::sync::Arc;

use lmssh::session::CommandExecutor;
use tokio::sync::Mutex;

#[tokio::test]
async fn executor_runs_commands_serially() {
    let executor = Arc::new(CommandExecutor::new());
    let log = Arc::new(Mutex::new(Vec::<String>::new()));

    let ex1 = executor.clone();
    let log1 = log.clone();
    let t1 = tokio::spawn(async move {
        ex1.run(|| async move {
            log1.lock().await.push("start-1".to_string());
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            log1.lock().await.push("end-1".to_string());
        })
        .await;
    });

    let ex2 = executor.clone();
    let log2 = log.clone();
    let t2 = tokio::spawn(async move {
        ex2.run(|| async move {
            log2.lock().await.push("start-2".to_string());
            log2.lock().await.push("end-2".to_string());
        })
        .await;
    });

    t1.await.unwrap();
    t2.await.unwrap();

    let got = log.lock().await.clone();
    assert_eq!(got, vec!["start-1", "end-1", "start-2", "end-2"]);
}
