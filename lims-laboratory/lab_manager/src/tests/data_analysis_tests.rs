#[cfg(test)]
mod data_analysis_tests {
    use serde_json::json;

    #[test]
    fn test_statistical_calculations() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];

        let sum: f64 = values.iter().sum();
        let mean = sum / values.len() as f64;

        assert_eq!(sum, 15.0);
        assert_eq!(mean, 3.0);
        assert_eq!(values.len(), 5);
    }

    #[test]
    fn test_quality_metrics() {
        let quality_scores = vec![8.5, 9.0, 7.8, 8.2, 9.1];

        let average: f64 = quality_scores.iter().sum::<f64>() / quality_scores.len() as f64;
        let max_score = quality_scores.iter().fold(0.0f64, |acc, &x| acc.max(x));
        let min_score = quality_scores.iter().fold(10.0f64, |acc, &x| acc.min(x));

        assert!((average - 8.52).abs() < 0.01);
        assert_eq!(max_score, 9.1);
        assert_eq!(min_score, 7.8);
    }

    #[test]
    fn test_data_aggregation() {
        let sample_data = json!({
            "samples": [
                {"type": "DNA", "concentration": 120.5},
                {"type": "DNA", "concentration": 95.0},
                {"type": "RNA", "concentration": 85.0},
                {"type": "RNA", "concentration": 102.0}
            ]
        });

        let samples = sample_data["samples"].as_array().unwrap();
        assert_eq!(samples.len(), 4);

        // Count by type
        let dna_count = samples.iter().filter(|s| s["type"] == "DNA").count();
        let rna_count = samples.iter().filter(|s| s["type"] == "RNA").count();

        assert_eq!(dna_count, 2);
        assert_eq!(rna_count, 2);
    }

    #[test]
    fn test_trend_analysis() {
        let monthly_counts = vec![150, 175, 160, 190, 210];

        // Check if last value is greater than first
        let growth = monthly_counts.last().unwrap() - monthly_counts.first().unwrap();
        let growth_rate = growth as f64 / *monthly_counts.first().unwrap() as f64 * 100.0;

        assert!(growth > 0, "Should show positive growth");
        assert_eq!(growth_rate, 40.0); // (210-150)/150*100
    }

    #[test]
    fn test_report_data_structure() {
        let report = json!({
            "total_samples": 145,
            "passed_qc": 138,
            "failed_qc": 7,
            "success_rate": 95.17
        });

        let total = report["total_samples"].as_i64().unwrap();
        let passed = report["passed_qc"].as_i64().unwrap();
        let failed = report["failed_qc"].as_i64().unwrap();

        assert_eq!(total, passed + failed);
        assert_eq!(total, 145);

        let success_rate = (passed as f64 / total as f64) * 100.0;
        assert!((success_rate - 95.17).abs() < 0.01);
    }

    #[test]
    fn test_chart_data_preparation() {
        let data_points = vec![("DNA", 65), ("RNA", 50), ("Protein", 30)];

        let labels: Vec<String> = data_points
            .iter()
            .map(|(label, _)| label.to_string())
            .collect();

        let values: Vec<i32> = data_points.iter().map(|(_, value)| *value).collect();

        assert_eq!(labels.len(), values.len());
        assert_eq!(labels.len(), 3);
        assert!(values.iter().all(|&v| v > 0));
    }
}
