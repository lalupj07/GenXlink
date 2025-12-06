use anyhow::Result;
use clap::Parser;
use std::sync::Arc;
use tracing::{info, error, warn};

use genxlink_tests::{TestRunner, TestConfig};

#[derive(Parser, Debug)]
#[command(name = "genxlink-test-runner")]
#[command(about = "GenXLink Comprehensive Test Suite Runner")]
#[command(version)]
pub struct Args {
    /// Test suite to run
    #[arg(short, long, default_value = "all")]
    suite: String,

    /// Configuration file
    #[arg(short, long, default_value = "test-config.toml")]
    config: String,

    /// Log level
    #[arg(short, long, default_value = "info")]
    log_level: String,

    /// Run tests in parallel
    #[arg(long)]
    parallel: bool,

    /// Timeout for individual tests (seconds)
    #[arg(long, default_value = "30")]
    timeout: u64,

    /// Generate HTML report
    #[arg(long)]
    html_report: bool,

    /// Output directory for reports
    #[arg(long, default_value = "./test-reports")]
    output_dir: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(match args.log_level.as_str() {
            "debug" => tracing::Level::DEBUG,
            "info" => tracing::Level::INFO,
            "warn" => tracing::Level::WARN,
            "error" => tracing::Level::ERROR,
            _ => tracing::Level::INFO,
        })
        .init();

    info!("Starting GenXLink Test Suite Runner v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let config = load_config(&args).await?;
    
    // Create test runner
    let test_runner = TestRunner::new(config);
    
    // Setup test environment
    test_runner.setup().await?;
    
    // Run the specified test suite
    let results = match test_runner.run_test_suite(&args.suite).await {
        Ok(results) => results,
        Err(e) => {
            error!("Test suite failed: {}", e);
            test_runner.teardown().await?;
            std::process::exit(1);
        }
    };
    
    // Generate reports if requested
    if args.html_report {
        generate_html_report(&results, &args.output_dir).await?;
    }
    
    // Teardown test environment
    test_runner.teardown().await?;
    
    // Exit with appropriate code
    let failed_count = results.iter().filter(|r| !r.passed).count();
    if failed_count > 0 {
        error!("{} tests failed", failed_count);
        std::process::exit(1);
    }
    
    info!("All tests passed successfully!");
    Ok(())
}

async fn load_config(args: &Args) -> Result<TestConfig> {
    let mut config = TestConfig::default();
    
    // Override with command line arguments
    config.log_level = args.log_level.clone();
    config.parallel_tests = args.parallel;
    config.timeout_seconds = args.timeout;
    
    // Load from file if it exists
    if std::path::Path::new(&args.config).exists() {
        info!("Loading test configuration from: {}", args.config);
        let settings = config::Config::builder()
            .add_source(config::File::with_name(&args.config))
            .build()?;
        
        // Override config with file settings
        if let Ok(database_url) = settings.get_string("database.url") {
            config.database_url = database_url;
        }
        if let Ok(redis_url) = settings.get_string("redis.url") {
            config.redis_url = redis_url;
        }
        if let Ok(api_endpoint) = settings.get_string("api.endpoint") {
            config.api_endpoint = api_endpoint;
        }
        if let Ok(relay_endpoint) = settings.get_string("relay.endpoint") {
            config.relay_endpoint = relay_endpoint;
        }
        if let Ok(signaling_endpoint) = settings.get_string("signaling.endpoint") {
            config.signaling_endpoint = signaling_endpoint;
        }
        if let Ok(test_data_dir) = settings.get_string("test.data_dir") {
            config.test_data_dir = test_data_dir;
        }
        if let Ok(parallel) = settings.get_bool("test.parallel") {
            config.parallel_tests = parallel;
        }
        if let Ok(timeout) = settings.get::<u64>("test.timeout") {
            config.timeout_seconds = timeout;
        }
    } else {
        warn!("Test configuration file not found: {}, using defaults", args.config);
    }
    
    info!("Loaded test configuration:");
    info!("  Database: {}", config.database_url);
    info!("  Redis: {}", config.redis_url);
    info!("  API Endpoint: {}", config.api_endpoint);
    info!("  Relay Endpoint: {}", config.relay_endpoint);
    info!("  Signaling Endpoint: {}", config.signaling_endpoint);
    info!("  Test Data Dir: {}", config.test_data_dir);
    info!("  Parallel Tests: {}", config.parallel_tests);
    info!("  Timeout: {}s", config.timeout_seconds);
    
    Ok(config)
}

async fn generate_html_report(results: &[genxlink_tests::TestResult], output_dir: &str) -> Result<()> {
    info!("Generating HTML report in: {}", output_dir);
    
    // Create output directory
    std::fs::create_dir_all(output_dir)?;
    
    // Generate HTML report
    let html_content = generate_html_content(results)?;
    
    // Write report file
    let report_path = std::path::Path::new(output_dir).join("test-report.html");
    std::fs::write(&report_path, html_content)?;
    
    // Generate JSON report
    let json_content = serde_json::to_string_pretty(results)?;
    let json_path = std::path::Path::new(output_dir).join("test-results.json");
    std::fs::write(&json_path, json_content)?;
    
    info!("HTML report generated: {}", report_path.display());
    info!("JSON report generated: {}", json_path.display());
    
    Ok(())
}

fn generate_html_content(results: &[genxlink_tests::TestResult]) -> Result<String> {
    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.len() - passed;
    let success_rate = (passed as f64 / results.len() as f64) * 100.0;
    
    let total_duration: std::time::Duration = results.iter().map(|r| r.duration).sum();
    
    let mut test_rows = String::new();
    for result in results {
        let status_class = if result.passed { "success" } else { "failed" };
        let status_icon = if result.passed { "✅" } else { "❌" };
        let error_msg = result.error_message.as_deref().unwrap_or("");
        
        test_rows.push_str(&format!(
            r#"
            <tr class="{}">
                <td>{}</td>
                <td>{}</td>
                <td>{:.2}s</td>
                <td>{}</td>
            </tr>
            "#,
            status_class, status_icon, result.test_name, 
            result.duration.as_secs_f64(), error_msg
        ));
    }
    
    let html = format!(
        r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>GenXLink Test Report</title>
            <style>
                body {{
                    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                    margin: 0;
                    padding: 20px;
                    background-color: #f5f5f5;
                }}
                .container {{
                    max-width: 1200px;
                    margin: 0 auto;
                    background: white;
                    border-radius: 8px;
                    box-shadow: 0 2px 10px rgba(0,0,0,0.1);
                    overflow: hidden;
                }}
                .header {{
                    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                    color: white;
                    padding: 30px;
                    text-align: center;
                }}
                .header h1 {{
                    margin: 0;
                    font-size: 2.5em;
                    font-weight: 300;
                }}
                .header p {{
                    margin: 10px 0 0 0;
                    opacity: 0.9;
                }}
                .summary {{
                    display: grid;
                    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
                    gap: 20px;
                    padding: 30px;
                    background: #fafafa;
                }}
                .metric {{
                    text-align: center;
                    padding: 20px;
                    background: white;
                    border-radius: 8px;
                    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
                }}
                .metric-value {{
                    font-size: 2em;
                    font-weight: bold;
                    color: #333;
                }}
                .metric-label {{
                    color: #666;
                    margin-top: 5px;
                }}
                .success {{ color: #28a745; }}
                .failed {{ color: #dc3545; }}
                .results {{
                    padding: 30px;
                }}
                .results h2 {{
                    margin-top: 0;
                    color: #333;
                }}
                table {{
                    width: 100%;
                    border-collapse: collapse;
                    margin-top: 20px;
                }}
                th, td {{
                    padding: 12px;
                    text-align: left;
                    border-bottom: 1px solid #ddd;
                }}
                th {{
                    background-color: #f8f9fa;
                    font-weight: 600;
                    color: #333;
                }}
                .success {{
                    background-color: #d4edda;
                }}
                .failed {{
                    background-color: #f8d7da;
                }}
                .timestamp {{
                    text-align: center;
                    color: #666;
                    padding: 20px;
                    font-size: 0.9em;
                }}
            </style>
        </head>
        <body>
            <div class="container">
                <div class="header">
                    <h1>GenXLink Test Report</h1>
                    <p>Generated on {}</p>
                </div>
                
                <div class="summary">
                    <div class="metric">
                        <div class="metric-value">{}</div>
                        <div class="metric-label">Total Tests</div>
                    </div>
                    <div class="metric">
                        <div class="metric-value success">{}</div>
                        <div class="metric-label">Passed</div>
                    </div>
                    <div class="metric">
                        <div class="metric-value failed">{}</div>
                        <div class="metric-label">Failed</div>
                    </div>
                    <div class="metric">
                        <div class="metric-value">{:.1}%</div>
                        <div class="metric-label">Success Rate</div>
                    </div>
                    <div class="metric">
                        <div class="metric-value">{:.2}s</div>
                        <div class="metric-label">Total Duration</div>
                    </div>
                </div>
                
                <div class="results">
                    <h2>Test Results</h2>
                    <table>
                        <thead>
                            <tr>
                                <th>Status</th>
                                <th>Test Name</th>
                                <th>Duration</th>
                                <th>Error Message</th>
                            </tr>
                        </thead>
                        <tbody>
                            {}
                        </tbody>
                    </table>
                </div>
                
                <div class="timestamp">
                    Report generated by GenXLink Test Runner v{}
                </div>
            </div>
        </body>
        </html>
        "#,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
        results.len(),
        passed,
        failed,
        success_rate,
        total_duration.as_secs_f64(),
        test_rows,
        env!("CARGO_PKG_VERSION")
    );
    
    Ok(html)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_config_loading() {
        let args = Args {
            suite: "integration".to_string(),
            config: "nonexistent.toml".to_string(),
            log_level: "debug".to_string(),
            parallel: true,
            timeout: 60,
            html_report: false,
            output_dir: "./reports".to_string(),
        };

        let config = load_config(&args).await.unwrap();
        
        assert_eq!(config.log_level, "debug");
        assert!(config.parallel_tests);
        assert_eq!(config.timeout_seconds, 60);
    }

    #[test]
    fn test_html_report_generation() {
        let results = vec![
            genxlink_tests::TestResult::passed("test1".to_string(), std::time::Duration::from_millis(100)),
            genxlink_tests::TestResult::failed("test2".to_string(), std::time::Duration::from_millis(200), "Error".to_string()),
        ];

        let html = generate_html_content(&results).unwrap();
        
        assert!(html.contains("GenXLink Test Report"));
        assert!(html.contains("test1"));
        assert!(html.contains("test2"));
        assert!(html.contains("✅"));
        assert!(html.contains("❌"));
    }
}
