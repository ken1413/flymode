use crate::config::{ActionType, AppConfig, ScheduleRule, TargetType};
use chrono::{DateTime, Datelike, Local, NaiveTime};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{error, info, warn};

pub struct Scheduler {
    config: Arc<RwLock<AppConfig>>,
    running: Arc<RwLock<bool>>,
    last_executed: Arc<RwLock<Vec<(String, bool)>>>,
}

impl Scheduler {
    pub fn new(config: Arc<RwLock<AppConfig>>) -> Self {
        Self {
            config,
            running: Arc::new(RwLock::new(false)),
            last_executed: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn start(&self) {
        let mut running = self.running.write().await;
        if *running {
            return;
        }
        *running = true;
        drop(running);

        let config = self.config.clone();
        let running_clone = self.running.clone();
        let last_executed = self.last_executed.clone();

        tokio::spawn(async move {
            let check_interval = {
                let cfg = config.read().await;
                Duration::from_secs(cfg.check_interval_seconds)
            };
            let mut ticker = interval(check_interval);

            loop {
                ticker.tick().await;

                let is_running = *running_clone.read().await;
                if !is_running {
                    break;
                }

                let cfg = config.read().await;
                let now = Local::now();

                for rule in &cfg.rules {
                    if !rule.enabled {
                        continue;
                    }

                    if let Some(action) = should_execute_rule(rule, &now, &last_executed).await {
                        info!("Executing rule: {} - {:?}", rule.name, action);
                        if let Err(e) = execute_action(rule.target.clone(), action).await {
                            error!("Failed to execute rule {}: {}", rule.name, e);
                        }
                    }
                }
            }
        });
    }

}

async fn should_execute_rule(
    rule: &ScheduleRule,
    now: &DateTime<Local>,
    last_executed: &Arc<RwLock<Vec<(String, bool)>>>,
) -> Option<ActionType> {
    let current_time = now.time();
    let current_weekday = now.weekday();

    let weekday_num = current_weekday.num_days_from_monday() as u8;
    if !rule.days.contains(&weekday_num) {
        return None;
    }

    let start_time = parse_time(&rule.start_time)?;
    let end_time = rule.end_time.as_ref().and_then(|t| parse_time(t));

    let in_time_range = if let Some(end) = end_time {
        if start_time <= end {
            current_time >= start_time && current_time <= end
        } else {
            current_time >= start_time || current_time <= end
        }
    } else {
        let diff = current_time.signed_duration_since(start_time).num_seconds().abs();
        diff < 60
    };

    if !in_time_range {
        let mut last = last_executed.write().await;
        last.retain(|(id, _)| id != &rule.id);
        return None;
    }

    let action_type = rule.action.clone();
    let is_start = match &rule.end_time {
        Some(_) => {
            let diff = current_time.signed_duration_since(start_time).num_seconds().abs();
            diff < 60
        }
        None => true,
    };

    let mut last = last_executed.write().await;
    let already_executed = last.iter().any(|(id, start)| id == &rule.id && *start == is_start);

    if already_executed {
        return None;
    }

    last.push((rule.id.clone(), is_start));
    Some(action_type)
}

fn parse_time(time_str: &str) -> Option<NaiveTime> {
    NaiveTime::parse_from_str(time_str, "%H:%M").ok()
}

async fn execute_action(target: TargetType, action: ActionType) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match target {
        TargetType::Wifi => {
            let cmd = get_wifi_command(&action)?;
            execute_command(&cmd).await?;
        }
        TargetType::Bluetooth => {
            let cmd = get_bluetooth_command(&action)?;
            execute_command(&cmd).await?;
        }
        TargetType::AirplaneMode => {
            let cmd = get_airplane_command(&action)?;
            execute_command(&cmd).await?;
        }
        TargetType::CustomCommand => {
            warn!("Custom command requires command string");
        }
    }
    Ok(())
}

pub fn get_wifi_command(action: &ActionType) -> Result<String, String> {
    match action {
        ActionType::Enable => Ok("nmcli radio wifi on".to_string()),
        ActionType::Disable => Ok("nmcli radio wifi off".to_string()),
        ActionType::Toggle => Ok("nmcli radio wifi".to_string()),
        _ => Err("Invalid action for WiFi".to_string()),
    }
}

pub fn get_bluetooth_command(action: &ActionType) -> Result<String, String> {
    match action {
        ActionType::Enable => Ok("rfkill unblock bluetooth".to_string()),
        ActionType::Disable => Ok("rfkill block bluetooth".to_string()),
        ActionType::Toggle => Ok("rfkill toggle bluetooth".to_string()),
        _ => Err("Invalid action for Bluetooth".to_string()),
    }
}

pub fn get_airplane_command(action: &ActionType) -> Result<String, String> {
    match action {
        ActionType::Enable => Ok("rfkill block all".to_string()),
        ActionType::Disable => Ok("rfkill unblock all".to_string()),
        _ => Err("Invalid action for Airplane Mode".to_string()),
    }
}

pub async fn execute_command(cmd: &str) -> Result<String, String> {
    info!("Executing command: {}", cmd);
    
    #[cfg(target_os = "windows")]
    let output = tokio::process::Command::new("cmd")
        .args(["/C", cmd])
        .output()
        .await
        .map_err(|e| e.to_string())?;
    
    #[cfg(not(target_os = "windows"))]
    let output = tokio::process::Command::new("sh")
        .args(["-c", cmd])
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

pub async fn execute_custom_command(cmd: &str) -> Result<String, String> {
    execute_command(cmd).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, TimeZone};
    use pretty_assertions::assert_eq;

    /// Helper: build a DateTime<Local> from y/m/d h:m:s
    fn make_datetime(year: i32, month: u32, day: u32, hour: u32, min: u32, sec: u32) -> DateTime<Local> {
        let naive = NaiveDate::from_ymd_opt(year, month, day)
            .unwrap()
            .and_hms_opt(hour, min, sec)
            .unwrap();
        Local.from_local_datetime(&naive).single().unwrap()
    }

    /// Helper: build a ScheduleRule with sensible defaults
    fn make_rule(id: &str, start: &str, end: Option<&str>, days: Vec<u8>) -> ScheduleRule {
        ScheduleRule {
            id: id.to_string(),
            name: "Test Rule".to_string(),
            enabled: true,
            action: ActionType::Enable,
            target: TargetType::Wifi,
            start_time: start.to_string(),
            end_time: end.map(|s| s.to_string()),
            days,
            command: None,
        }
    }

    fn make_last_executed() -> Arc<RwLock<Vec<(String, bool)>>> {
        Arc::new(RwLock::new(Vec::new()))
    }

    // ── parse_time ──────────────────────────────────────────────────

    #[test]
    fn test_parse_time_valid() {
        assert_eq!(parse_time("09:00"), Some(NaiveTime::from_hms_opt(9, 0, 0).unwrap()));
        assert_eq!(parse_time("23:59"), Some(NaiveTime::from_hms_opt(23, 59, 0).unwrap()));
        assert_eq!(parse_time("00:00"), Some(NaiveTime::from_hms_opt(0, 0, 0).unwrap()));
    }

    #[test]
    fn test_parse_time_invalid() {
        assert_eq!(parse_time("invalid"), None);
        assert_eq!(parse_time("25:00"), None);
        assert_eq!(parse_time(""), None);
    }

    // ── command generation ──────────────────────────────────────────

    #[test]
    fn test_wifi_enable() {
        assert_eq!(get_wifi_command(&ActionType::Enable), Ok("nmcli radio wifi on".to_string()));
    }

    #[test]
    fn test_wifi_disable() {
        assert_eq!(get_wifi_command(&ActionType::Disable), Ok("nmcli radio wifi off".to_string()));
    }

    #[test]
    fn test_wifi_toggle() {
        assert_eq!(get_wifi_command(&ActionType::Toggle), Ok("nmcli radio wifi".to_string()));
    }

    #[test]
    fn test_wifi_invalid_action() {
        assert!(get_wifi_command(&ActionType::RunCommand).is_err());
    }

    #[test]
    fn test_bluetooth_enable() {
        assert_eq!(get_bluetooth_command(&ActionType::Enable), Ok("rfkill unblock bluetooth".to_string()));
    }

    #[test]
    fn test_bluetooth_disable() {
        assert_eq!(get_bluetooth_command(&ActionType::Disable), Ok("rfkill block bluetooth".to_string()));
    }

    #[test]
    fn test_bluetooth_toggle() {
        assert_eq!(get_bluetooth_command(&ActionType::Toggle), Ok("rfkill toggle bluetooth".to_string()));
    }

    #[test]
    fn test_bluetooth_invalid_action() {
        assert!(get_bluetooth_command(&ActionType::RunCommand).is_err());
    }

    #[test]
    fn test_airplane_enable() {
        assert_eq!(get_airplane_command(&ActionType::Enable), Ok("rfkill block all".to_string()));
    }

    #[test]
    fn test_airplane_disable() {
        assert_eq!(get_airplane_command(&ActionType::Disable), Ok("rfkill unblock all".to_string()));
    }

    #[test]
    fn test_airplane_toggle_invalid() {
        assert!(get_airplane_command(&ActionType::Toggle).is_err());
    }

    #[test]
    fn test_airplane_run_command_invalid() {
        assert!(get_airplane_command(&ActionType::RunCommand).is_err());
    }

    // ── should_execute_rule: weekday filtering ──────────────────────

    #[tokio::test]
    async fn test_weekday_no_match_returns_none() {
        // 2026-02-27 is Friday → num_days_from_monday = 4
        let now = make_datetime(2026, 2, 27, 9, 0, 0);
        let rule = make_rule("r1", "09:00", None, vec![0, 1, 2]); // Mon/Tue/Wed
        let last = make_last_executed();

        assert!(should_execute_rule(&rule, &now, &last).await.is_none());
    }

    #[tokio::test]
    async fn test_weekday_match_executes() {
        let now = make_datetime(2026, 2, 27, 9, 0, 0); // Friday=4
        let rule = make_rule("r1", "09:00", None, vec![4]);
        let last = make_last_executed();

        assert_eq!(
            should_execute_rule(&rule, &now, &last).await,
            Some(ActionType::Enable)
        );
    }

    #[tokio::test]
    async fn test_all_days_enabled() {
        let now = make_datetime(2026, 2, 27, 9, 0, 0);
        let rule = make_rule("r1", "09:00", None, vec![0, 1, 2, 3, 4, 5, 6]);
        let last = make_last_executed();

        assert_eq!(
            should_execute_rule(&rule, &now, &last).await,
            Some(ActionType::Enable)
        );
    }

    // ── should_execute_rule: start-time-only (no end_time) ──────────

    #[tokio::test]
    async fn test_start_only_within_60s_window() {
        let now = make_datetime(2026, 2, 27, 9, 0, 30); // 30s after start
        let rule = make_rule("r1", "09:00", None, vec![4]);
        let last = make_last_executed();

        assert_eq!(
            should_execute_rule(&rule, &now, &last).await,
            Some(ActionType::Enable)
        );
    }

    #[tokio::test]
    async fn test_start_only_outside_60s_window() {
        let now = make_datetime(2026, 2, 27, 9, 2, 0); // 2 min after start
        let rule = make_rule("r1", "09:00", None, vec![4]);
        let last = make_last_executed();

        assert!(should_execute_rule(&rule, &now, &last).await.is_none());
    }

    #[tokio::test]
    async fn test_start_only_exactly_at_boundary() {
        let now = make_datetime(2026, 2, 27, 9, 0, 59); // 59s → within window
        let rule = make_rule("r1", "09:00", None, vec![4]);
        let last = make_last_executed();

        assert_eq!(
            should_execute_rule(&rule, &now, &last).await,
            Some(ActionType::Enable)
        );
    }

    // ── should_execute_rule: time range (same day) ──────────────────

    #[tokio::test]
    async fn test_range_same_day_in_range() {
        let now = make_datetime(2026, 2, 27, 12, 0, 0); // midday
        let rule = make_rule("r1", "09:00", Some("17:00"), vec![4]);
        let last = make_last_executed();

        assert_eq!(
            should_execute_rule(&rule, &now, &last).await,
            Some(ActionType::Enable)
        );
    }

    #[tokio::test]
    async fn test_range_same_day_before_start() {
        let now = make_datetime(2026, 2, 27, 8, 0, 0);
        let rule = make_rule("r1", "09:00", Some("17:00"), vec![4]);
        let last = make_last_executed();

        assert!(should_execute_rule(&rule, &now, &last).await.is_none());
    }

    #[tokio::test]
    async fn test_range_same_day_after_end() {
        let now = make_datetime(2026, 2, 27, 20, 0, 0);
        let rule = make_rule("r1", "09:00", Some("17:00"), vec![4]);
        let last = make_last_executed();

        assert!(should_execute_rule(&rule, &now, &last).await.is_none());
    }

    #[tokio::test]
    async fn test_range_at_start_boundary() {
        let now = make_datetime(2026, 2, 27, 9, 0, 0);
        let rule = make_rule("r1", "09:00", Some("17:00"), vec![4]);
        let last = make_last_executed();

        assert_eq!(
            should_execute_rule(&rule, &now, &last).await,
            Some(ActionType::Enable)
        );
    }

    #[tokio::test]
    async fn test_range_at_end_boundary() {
        let now = make_datetime(2026, 2, 27, 17, 0, 0);
        let rule = make_rule("r1", "09:00", Some("17:00"), vec![4]);
        let last = make_last_executed();

        assert_eq!(
            should_execute_rule(&rule, &now, &last).await,
            Some(ActionType::Enable)
        );
    }

    // ── should_execute_rule: midnight crossover ─────────────────────

    #[tokio::test]
    async fn test_midnight_crossover_after_start() {
        // Rule: 23:00 → 02:00; now is 23:30 on Friday
        let now = make_datetime(2026, 2, 27, 23, 30, 0); // Friday=4
        let rule = make_rule("r1", "23:00", Some("02:00"), vec![4]);
        let last = make_last_executed();

        assert_eq!(
            should_execute_rule(&rule, &now, &last).await,
            Some(ActionType::Enable)
        );
    }

    #[tokio::test]
    async fn test_midnight_crossover_after_midnight() {
        // Rule: 23:00 → 02:00; now is 00:30 on Saturday
        let now = make_datetime(2026, 2, 28, 0, 30, 0); // Saturday=5
        let rule = make_rule("r1", "23:00", Some("02:00"), vec![5]);
        let last = make_last_executed();

        assert_eq!(
            should_execute_rule(&rule, &now, &last).await,
            Some(ActionType::Enable)
        );
    }

    #[tokio::test]
    async fn test_midnight_crossover_outside_range() {
        let now = make_datetime(2026, 2, 28, 15, 0, 0); // Saturday afternoon
        let rule = make_rule("r1", "23:00", Some("02:00"), vec![5]);
        let last = make_last_executed();

        assert!(should_execute_rule(&rule, &now, &last).await.is_none());
    }

    // ── should_execute_rule: deduplication ───────────────────────────

    #[tokio::test]
    async fn test_dedup_prevents_double_execution() {
        let now = make_datetime(2026, 2, 27, 9, 0, 0);
        let rule = make_rule("r1", "09:00", None, vec![4]);
        let last = make_last_executed();

        // First: executes
        assert!(should_execute_rule(&rule, &now, &last).await.is_some());
        // Second: deduped
        assert!(should_execute_rule(&rule, &now, &last).await.is_none());
    }

    #[tokio::test]
    async fn test_different_rules_not_deduped() {
        let now = make_datetime(2026, 2, 27, 9, 0, 0);
        let rule_a = make_rule("r1", "09:00", None, vec![4]);
        let rule_b = make_rule("r2", "09:00", None, vec![4]);
        let last = make_last_executed();

        assert!(should_execute_rule(&rule_a, &now, &last).await.is_some());
        assert!(should_execute_rule(&rule_b, &now, &last).await.is_some());
    }

    #[tokio::test]
    async fn test_out_of_range_clears_dedup_state() {
        let rule = make_rule("r1", "09:00", None, vec![4]);
        let last = make_last_executed();

        // Execute at 09:00
        let now = make_datetime(2026, 2, 27, 9, 0, 0);
        assert!(should_execute_rule(&rule, &now, &last).await.is_some());

        // Out of range at 09:05 → clears last_executed for this rule
        let now = make_datetime(2026, 2, 27, 9, 5, 0);
        assert!(should_execute_rule(&rule, &now, &last).await.is_none());

        // Verify state was cleared
        let entries = last.read().await;
        assert!(entries.is_empty(), "last_executed should be cleared after out-of-range check");
    }

    // ── should_execute_rule: invalid input ──────────────────────────

    #[tokio::test]
    async fn test_invalid_start_time_returns_none() {
        let now = make_datetime(2026, 2, 27, 9, 0, 0);
        let rule = make_rule("r1", "invalid", None, vec![4]);
        let last = make_last_executed();

        assert!(should_execute_rule(&rule, &now, &last).await.is_none());
    }

    #[tokio::test]
    async fn test_empty_days_returns_none() {
        let now = make_datetime(2026, 2, 27, 9, 0, 0);
        let rule = make_rule("r1", "09:00", None, vec![]);
        let last = make_last_executed();

        assert!(should_execute_rule(&rule, &now, &last).await.is_none());
    }
}
