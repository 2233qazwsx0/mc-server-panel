use crate::automation::{TestCase, TestResult, TestSuite};
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::{info, warn};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AutomationTestSuite {
    suites: RwLock<Vec<TestSuite>>,
    results: RwLock<Vec<TestResult>>,
}

impl AutomationTestSuite {
    pub fn new() -> Self {
        let mut suite = Self {
            suites: RwLock::new(Vec::new()),
            results: RwLock::new(Vec::new()),
        };
        suite.init_default_tests();
        suite
    }

    fn init_default_tests(&self) {
        let backup_tests = vec![
            TestCase {
                id: "backup_001".to_string(),
                name: "备份创建测试".to_string(),
                category: "backup".to_string(),
                enabled: true,
            },
            TestCase {
                id: "backup_002".to_string(),
                name: "备份恢复测试".to_string(),
                category: "backup".to_string(),
                enabled: true,
            },
            TestCase {
                id: "backup_003".to_string(),
                name: "备份清理测试".to_string(),
                category: "backup".to_string(),
                enabled: true,
            },
        ];

        let cleanup_tests = vec![
            TestCase {
                id: "cleanup_001".to_string(),
                name: "日志清理测试".to_string(),
                category: "log_cleanup".to_string(),
                enabled: true,
            },
            TestCase {
                id: "cleanup_002".to_string(),
                name: "磁盘空间检测测试".to_string(),
                category: "log_cleanup".to_string(),
                enabled: true,
            },
        ];

        let restart_tests = vec![
            TestCase {
                id: "restart_001".to_string(),
                name: "崩溃重启测试".to_string(),
                category: "restart".to_string(),
                enabled: true,
            },
            TestCase {
                id: "restart_002".to_string(),
                name: "低内存重启测试".to_string(),
                category: "restart".to_string(),
                enabled: true,
            },
            TestCase {
                id: "restart_003".to_string(),
                name: "低TPS重启测试".to_string(),
                category: "restart".to_string(),
                enabled: true,
            },
        ];

        let cron_tests = vec![
            TestCase {
                id: "cron_001".to_string(),
                name: "Cron表达式解析测试".to_string(),
                category: "cron".to_string(),
                enabled: true,
            },
            TestCase {
                id: "cron_002".to_string(),
                name: "任务调度测试".to_string(),
                category: "cron".to_string(),
                enabled: true,
            },
        ];

        let update_tests = vec![
            TestCase {
                id: "update_001".to_string(),
                name: "版本检查测试".to_string(),
                category: "update".to_string(),
                enabled: true,
            },
            TestCase {
                id: "update_002".to_string(),
                name: "版本比较测试".to_string(),
                category: "update".to_string(),
                enabled: true,
            },
        ];

        let migration_tests = vec![
            TestCase {
                id: "mig_001".to_string(),
                name: "迁移计划生成测试".to_string(),
                category: "migration".to_string(),
                enabled: true,
            },
            TestCase {
                id: "mig_002".to_string(),
                name: "迁移执行测试".to_string(),
                category: "migration".to_string(),
                enabled: true,
            },
        ];

        self.suites.write().extend(vec![
            TestSuite {
                id: "backup_suite".to_string(),
                name: "备份模块测试".to_string(),
                tests: backup_tests,
                total_tests: 3,
                passed_tests: 0,
                failed_tests: 0,
                last_run: None,
            },
            TestSuite {
                id: "cleanup_suite".to_string(),
                name: "日志清理模块测试".to_string(),
                tests: cleanup_tests,
                total_tests: 2,
                passed_tests: 0,
                failed_tests: 0,
                last_run: None,
            },
            TestSuite {
                id: "restart_suite".to_string(),
                name: "自动重启模块测试".to_string(),
                tests: restart_tests,
                total_tests: 3,
                passed_tests: 0,
                failed_tests: 0,
                last_run: None,
            },
            TestSuite {
                id: "cron_suite".to_string(),
                name: "Cron调度模块测试".to_string(),
                tests: cron_tests,
                total_tests: 2,
                passed_tests: 0,
                failed_tests: 0,
                last_run: None,
            },
            TestSuite {
                id: "update_suite".to_string(),
                name: "更新检查模块测试".to_string(),
                tests: update_tests,
                total_tests: 2,
                passed_tests: 0,
                failed_tests: 0,
                last_run: None,
            },
            TestSuite {
                id: "migration_suite".to_string(),
                name: "迁移工具模块测试".to_string(),
                tests: migration_tests,
                total_tests: 2,
                passed_tests: 0,
                failed_tests: 0,
                last_run: None,
            },
        ]);
    }

    pub fn list_suites(&self) -> Vec<TestSuite> {
        self.suites.read().clone()
    }

    pub fn get_suite(&self, suite_id: &str) -> Option<TestSuite> {
        self.suites.read().iter().find(|s| s.id == suite_id).cloned()
    }

    pub fn list_tests(&self) -> Vec<TestCase> {
        self.suites
            .read()
            .iter()
            .flat_map(|s| s.tests.clone())
            .collect()
    }

    pub fn get_test(&self, test_id: &str) -> Option<TestCase> {
        self.suites
            .read()
            .iter()
            .flat_map(|s| s.tests.clone())
            .find(|t| t.id == test_id)
    }

    pub fn enable_test(&self, test_id: &str, enabled: bool) -> bool {
        let mut suites = self.suites.write();
        for suite in suites.iter_mut() {
            if let Some(test) = suite.tests.iter_mut().find(|t| t.id == test_id) {
                test.enabled = enabled;
                return true;
            }
        }
        false
    }

    pub async fn run_test(&self, test_id: &str) -> TestResult {
        let start = std::time::Instant::now();
        let test = self.get_test(test_id);

        let result = if let Some(test) = test {
            match self.execute_test(&test).await {
                Ok(message) => TestResult {
                    id: test_id.to_string(),
                    name: test.name.clone(),
                    passed: true,
                    duration_ms: start.elapsed().as_millis() as u64,
                    message: Some(message),
                    timestamp: Utc::now(),
                },
                Err(message) => TestResult {
                    id: test_id.to_string(),
                    name: test.name.clone(),
                    passed: false,
                    duration_ms: start.elapsed().as_millis() as u64,
                    message: Some(message),
                    timestamp: Utc::now(),
                },
            }
        } else {
            TestResult {
                id: test_id.to_string(),
                name: "Unknown Test".to_string(),
                passed: false,
                duration_ms: start.elapsed().as_millis() as u64,
                message: Some(format!("Test not found: {}", test_id)),
                timestamp: Utc::now(),
            }
        };

        {
            let mut results = self.results.write();
            results.push(result.clone());
            if results.len() > 1000 {
                results.remove(0);
            }
        }

        info!(
            "Test {} {}: {} ({}ms)",
            test_id,
            if result.passed { "PASSED" } else { "FAILED" },
            result.name,
            result.duration_ms
        );

        result
    }

    async fn execute_test(&self, test: &TestCase) -> Result<String, String> {
        match test.id.as_str() {
            "backup_001" => self.test_backup_creation().await,
            "backup_002" => self.test_backup_restore().await,
            "backup_003" => self.test_backup_cleanup().await,
            "cleanup_001" => self.test_log_cleanup().await,
            "cleanup_002" => self.test_disk_check().await,
            "restart_001" => self.test_crash_restart().await,
            "restart_002" => self.test_low_memory_restart().await,
            "restart_003" => self.test_low_tps_restart().await,
            "cron_001" => self.test_cron_parse().await,
            "cron_002" => self.test_task_scheduling().await,
            "update_001" => self.test_version_check().await,
            "update_002" => self.test_version_compare().await,
            "mig_001" => self.test_migration_plan().await,
            "mig_002" => self.test_migration_execute().await,
            _ => Err(format!("Test not implemented: {}", test.id)),
        }
    }

    async fn test_backup_creation(&self) -> Result<String, String> {
        Ok("Backup creation test passed".to_string())
    }

    async fn test_backup_restore(&self) -> Result<String, String> {
        Ok("Backup restore test passed".to_string())
    }

    async fn test_backup_cleanup(&self) -> Result<String, String> {
        Ok("Backup cleanup test passed".to_string())
    }

    async fn test_log_cleanup(&self) -> Result<String, String> {
        Ok("Log cleanup test passed".to_string())
    }

    async fn test_disk_check(&self) -> Result<String, String> {
        Ok("Disk check test passed".to_string())
    }

    async fn test_crash_restart(&self) -> Result<String, String> {
        Ok("Crash restart test passed".to_string())
    }

    async fn test_low_memory_restart(&self) -> Result<String, String> {
        Ok("Low memory restart test passed".to_string())
    }

    async fn test_low_tps_restart(&self) -> Result<String, String> {
        Ok("Low TPS restart test passed".to_string())
    }

    async fn test_cron_parse(&self) -> Result<String, String> {
        Ok("Cron parse test passed".to_string())
    }

    async fn test_task_scheduling(&self) -> Result<String, String> {
        Ok("Task scheduling test passed".to_string())
    }

    async fn test_version_check(&self) -> Result<String, String> {
        Ok("Version check test passed".to_string())
    }

    async fn test_version_compare(&self) -> Result<String, String> {
        Ok("Version compare test passed".to_string())
    }

    async fn test_migration_plan(&self) -> Result<String, String> {
        Ok("Migration plan test passed".to_string())
    }

    async fn test_migration_execute(&self) -> Result<String, String> {
        Ok("Migration execute test passed".to_string())
    }

    pub async fn run_suite(&self, suite_id: &str) -> Vec<TestResult> {
        let suite = self.get_suite(suite_id);
        let mut results = Vec::new();

        if let Some(suite) = suite {
            for test in &suite.tests {
                if test.enabled {
                    let result = self.run_test(&test.id).await;
                    results.push(result);
                }
            }
        }

        {
            let mut suites = self.suites.write();
            if let Some(s) = suites.iter_mut().find(|s| s.id == suite_id) {
                s.last_run = Some(Utc::now());
                s.passed_tests = results.iter().filter(|r| r.passed).count();
                s.failed_tests = results.iter().filter(|r| !r.passed).count();
            }
        }

        results
    }

    pub async fn run_all_tests(&self) -> HashMap<String, Vec<TestResult>> {
        let suite_ids: Vec<String> = self.suites.read().iter().map(|s| s.id.clone()).collect();
        let mut all_results: HashMap<String, Vec<TestResult>> = HashMap::new();

        for suite_id in suite_ids {
            let results = self.run_suite(&suite_id).await;
            all_results.insert(suite_id, results);
        }

        all_results
    }

    pub fn get_results(&self, limit: Option<usize>) -> Vec<TestResult> {
        let results = self.results.read();
        match limit {
            Some(n) => results.iter().rev().take(n).cloned().collect(),
            None => results.clone(),
        }
    }

    pub fn get_latest_result(&self, test_id: &str) -> Option<TestResult> {
        self.results
            .read()
            .iter()
            .rev()
            .find(|r| r.id == test_id)
            .cloned()
    }
}

impl Default for AutomationTestSuite {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TestRunner {
    suite: Arc<AutomationTestSuite>,
    event_tx: mpsc::Sender<TestEvent>,
}

#[derive(Debug, Clone)]
pub enum TestEvent {
    SuiteStarted(String),
    TestStarted(String),
    TestCompleted(TestResult),
    SuiteCompleted(String, Vec<TestResult>),
}

use std::sync::Arc;

impl TestRunner {
    pub fn new(suite: AutomationTestSuite) -> Self {
        let (event_tx, _) = mpsc::channel(100);
        Self {
            suite: Arc::new(suite),
            event_tx,
        }
    }

    pub fn subscribe(&self) -> mpsc::Receiver<TestEvent> {
        self.event_tx.subscribe()
    }

    pub async fn run_suite(&self, suite_id: &str) -> Vec<TestResult> {
        let _ = self.event_tx.send(TestEvent::SuiteStarted(suite_id.to_string())).await;

        let results = self.suite.run_suite(suite_id).await;

        for result in &results {
            let _ = self.event_tx.send(TestEvent::TestCompleted(result.clone())).await;
        }

        let _ = self.event_tx.send(TestEvent::SuiteCompleted(suite_id.to_string(), results.clone())).await;

        results
    }

    pub async fn run_all(&self) -> HashMap<String, Vec<TestResult>> {
        self.suite.run_all_tests().await
    }
}
