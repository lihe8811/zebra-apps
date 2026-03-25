use zebra_core::config::ScheduleConfig;

fn main() {
    let schedule = ScheduleConfig {
        job_name: "summarize-hourly".to_string(),
        cron: "0 * * * *".to_string(),
    };

    println!(
        "ops bootstrap placeholder. Future launchd support for {} on {}",
        schedule.job_name, schedule.cron
    );
}
