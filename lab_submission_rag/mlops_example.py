"""
TracSeq 2.0 MLOps Pipeline Example

Comprehensive demonstration of the complete MLOps pipeline including:
- Model registry and versioning
- Experiment tracking
- A/B testing
- Continuous learning
- Model monitoring
- Automated deployment
- Data pipeline
"""

import asyncio

import numpy as np
import pandas as pd
from sklearn.ensemble import RandomForestClassifier
from sklearn.metrics import accuracy_score, f1_score, precision_score, recall_score
from sklearn.model_selection import train_test_split

from mlops.ab_testing import ABTestConfig, ABTestInteraction, ABTestManager, TestType
from mlops.continuous_learning import (
    ContinuousLearningPipeline,
    RetrainingConfig,
    TriggerType,
)
from mlops.data_pipeline import DataPipeline, DataSchema, FeatureEngineeringConfig
from mlops.deployment_manager import (
    DeploymentConfig,
    DeploymentEnvironment,
    DeploymentStrategy,
    ModelDeploymentManager,
)
from mlops.experiment_tracker import (
    ExperimentConfig,
    ExperimentMetrics,
    ExperimentTracker,
)

# Import MLOps components
from mlops.model_registry import ModelMetadata, ModelRegistry, ModelStatus
from mlops.monitoring import (
    MetricThreshold,
    MetricType,
    ModelMonitor,
)


class TracSeqMLOpsDemo:
    """
    Complete demonstration of TracSeq 2.0 MLOps pipeline
    """

    def __init__(self) -> None:
        # Database URL (use appropriate database for production)
        self.database_url = "sqlite:///mlops_demo.db"

        # Initialize MLOps components
        self.model_registry = None
        self.experiment_tracker = None
        self.ab_test_manager = None
        self.continuous_learning = None
        self.model_monitor = None
        self.deployment_manager = None
        self.data_pipeline = None

    async def initialize_components(self) -> None:
        """Initialize all MLOps components"""
        print("🚀 Initializing MLOps components...")

        # Model Registry
        self.model_registry = ModelRegistry(
            registry_path="./mlops_data/model_registry", database_url=self.database_url
        )

        # Experiment Tracker
        self.experiment_tracker = ExperimentTracker(
            tracking_dir="./mlops_data/experiments", database_url=self.database_url
        )

        # A/B Testing
        self.ab_test_manager = ABTestManager(
            database_url=self.database_url, results_dir="./mlops_data/ab_tests"
        )

        # Continuous Learning
        self.continuous_learning = ContinuousLearningPipeline(
            database_url=self.database_url,
            data_dir="./mlops_data/continuous_learning",
            model_registry=self.model_registry,
            experiment_tracker=self.experiment_tracker,
        )

        # Model Monitor
        self.model_monitor = ModelMonitor(
            database_url=self.database_url, dashboard_dir="./mlops_data/dashboards"
        )

        # Deployment Manager
        self.deployment_manager = ModelDeploymentManager(
            database_url=self.database_url, container_registry_url="localhost:5000"
        )

        # Data Pipeline
        self.data_pipeline = DataPipeline(
            database_url=self.database_url, data_dir="./mlops_data/data_pipeline"
        )

        print("✅ All components initialized successfully!")

    async def demo_data_pipeline(self) -> None:
        """Demonstrate data pipeline functionality"""
        print("\n📊 === DATA PIPELINE DEMO ===")

        # Create sample data
        print("Creating sample lab data...")
        sample_data = self._create_sample_lab_data()
        sample_data.to_csv("./sample_lab_data.csv", index=False)

        # Define data schema
        schema = DataSchema(
            schema_id="",
            name="Lab Sample Data Schema",
            version="1.0",
            columns={
                "sample_id": {"type": "string", "nullable": False},
                "submission_date": {"type": "datetime", "nullable": False},
                "sample_type": {"type": "string", "nullable": False},
                "concentration": {"type": "float", "nullable": True},
                "quality_score": {"type": "float", "nullable": False},
                "processing_time": {"type": "float", "nullable": False},
                "approved": {"type": "boolean", "nullable": False},
            },
            description="Schema for laboratory sample data",
        )

        schema_id = await self.data_pipeline.create_schema(schema)
        print(f"✅ Created data schema: {schema_id}")

        # Define feature engineering configuration
        feature_config = FeatureEngineeringConfig(
            config_id="",
            name="Lab Data Feature Engineering",
            numeric_features=["concentration", "quality_score", "processing_time"],
            categorical_features=["sample_type"],
            scaling_method="standard",
            encoding_method="label",
            feature_selection_enabled=True,
            correlation_threshold=0.9,
        )

        feature_config_id = await self.data_pipeline.create_feature_config(feature_config)
        print(f"✅ Created feature config: {feature_config_id}")

        # Run data pipeline
        print("Running data pipeline...")
        run_id = await self.data_pipeline.run_pipeline(
            pipeline_id="lab_data_pipeline",
            input_data_path="./sample_lab_data.csv",
            schema_id=schema_id,
            feature_config_id=feature_config_id,
        )

        # Wait for pipeline to complete
        await asyncio.sleep(5)

        pipeline_run = await self.data_pipeline.get_pipeline_run(run_id)
        if pipeline_run:
            print(f"✅ Pipeline completed: {pipeline_run.status.value}")
            print(f"   Input rows: {pipeline_run.input_rows}")
            print(f"   Output rows: {pipeline_run.output_rows}")
            print(f"   Features: {pipeline_run.feature_count}")
            print(f"   Quality score: {pipeline_run.data_quality_score:.3f}")

        return pipeline_run.output_data_path if pipeline_run else None

    async def demo_experiment_tracking(self, processed_data_path: str) -> None:
        """Demonstrate experiment tracking"""
        print("\n🧪 === EXPERIMENT TRACKING DEMO ===")

        # Load processed data
        df = pd.read_csv(processed_data_path)
        X = df.drop("approved", axis=1, errors="ignore")
        y = df["approved"] if "approved" in df.columns else np.random.choice([0, 1], size=len(df))

        X_train, X_test, y_train, y_test = train_test_split(X, y, test_size=0.2, random_state=42)

        # Start experiment
        exp_config = ExperimentConfig(
            experiment_id="",
            name="Lab Sample Classification Experiment",
            description="Training a model to predict lab sample approval",
            model_type="random_forest",
            framework="sklearn",
            algorithm="RandomForestClassifier",
            dataset_name="lab_samples",
            hyperparameters={"n_estimators": 100, "max_depth": 10, "random_state": 42},
        )

        experiment_id = await self.experiment_tracker.start_experiment(exp_config)
        print(f"✅ Started experiment: {experiment_id}")

        # Train model
        print("Training model...")
        model = RandomForestClassifier(
            n_estimators=exp_config.hyperparameters["n_estimators"],
            max_depth=exp_config.hyperparameters["max_depth"],
            random_state=exp_config.hyperparameters["random_state"],
        )

        model.fit(X_train, y_train)

        # Make predictions and calculate metrics
        y_pred = model.predict(X_test)

        metrics = ExperimentMetrics(
            experiment_id=experiment_id,
            step=1,
            epoch=1,
            train_accuracy=model.score(X_train, y_train),
            val_accuracy=accuracy_score(y_test, y_pred),
            val_precision=precision_score(y_test, y_pred, average="weighted"),
            val_recall=recall_score(y_test, y_pred, average="weighted"),
            val_f1=f1_score(y_test, y_pred, average="weighted"),
        )

        await self.experiment_tracker.log_metrics(metrics)
        print(f"✅ Logged metrics - Accuracy: {metrics.val_accuracy:.3f}")

        # Log model artifact
        await self.experiment_tracker.log_artifact(
            name="trained_model",
            artifact_type="model",
            content=model,
            experiment_id=experiment_id,
            description="Trained Random Forest model",
        )

        # Complete experiment
        await self.experiment_tracker.complete_experiment(
            experiment_id=experiment_id,
            final_metrics={
                "accuracy": metrics.val_accuracy,
                "precision": metrics.val_precision,
                "recall": metrics.val_recall,
                "f1_score": metrics.val_f1,
            },
        )

        print("✅ Experiment completed successfully!")
        return model, experiment_id, metrics

    async def demo_model_registry(self, model, experiment_id: str, metrics) -> None:
        """Demonstrate model registry functionality"""
        print("\n📦 === MODEL REGISTRY DEMO ===")

        # Create model metadata
        metadata = ModelMetadata(
            model_id="lab_sample_classifier",
            version="1.0.0",
            name="Lab Sample Approval Classifier",
            description="Random Forest model for predicting lab sample approval",
            model_type="classification",
            framework="sklearn",
            accuracy=metrics.val_accuracy,
            precision=metrics.val_precision,
            recall=metrics.val_recall,
            f1_score=metrics.val_f1,
            confidence_score=0.95,
            training_data_hash="dummy_hash",
            training_duration_seconds=30.0,
            hyperparameters={"n_estimators": 100, "max_depth": 10},
            feature_columns=["feature_1", "feature_2", "feature_3"],
            status=ModelStatus.STAGING,
            created_by="mlops_demo",
        )

        # Register model
        model_id = await self.model_registry.register_model(
            model=model, metadata=metadata, config={"experiment_id": experiment_id}
        )

        print(f"✅ Registered model: {model_id}")

        # Promote model to production
        success = await self.model_registry.promote_model(
            "lab_sample_classifier", "1.0.0", ModelStatus.PRODUCTION
        )

        if success:
            print("✅ Model promoted to production")

        return model_id

    async def demo_monitoring(self, model_id: str) -> None:
        """Demonstrate model monitoring"""
        print("\n📈 === MODEL MONITORING DEMO ===")

        # Start monitoring
        await self.model_monitor.start_monitoring()

        # Create monitoring thresholds
        latency_threshold = MetricThreshold(
            threshold_id="",
            model_id="lab_sample_classifier",
            metric_name="prediction_latency",
            metric_type=MetricType.LATENCY,
            warning_threshold=500.0,  # 500ms
            critical_threshold=1000.0,  # 1000ms
            operator="gt",
            evaluation_window_minutes=5,
            alert_enabled=True,
            description="Monitor prediction latency",
        )

        threshold_id = await self.model_monitor.create_threshold(latency_threshold)
        print(f"✅ Created monitoring threshold: {threshold_id}")

        # Simulate some predictions and record metrics
        print("Simulating predictions and recording metrics...")
        for i in range(10):
            # Simulate prediction metrics
            latency = np.random.normal(200, 50)  # Normal latency around 200ms
            success = np.random.choice([True, False], p=[0.95, 0.05])
            confidence = np.random.uniform(0.7, 0.99)
            accuracy = np.random.uniform(0.85, 0.95)

            await self.model_monitor.record_prediction_metrics(
                model_id="lab_sample_classifier",
                model_version="1.0.0",
                prediction_latency_ms=latency,
                prediction_success=success,
                confidence_score=confidence,
                accuracy=accuracy,
                request_id=f"req_{i}",
            )

        print("✅ Recorded prediction metrics")

        # Get model health
        await asyncio.sleep(2)  # Wait for metrics to be processed
        health = await self.model_monitor.get_model_health("lab_sample_classifier")
        if health:
            print(f"✅ Model health: {health.overall_health} (score: {health.health_score:.3f})")

        # Generate dashboard
        dashboard_path = await self.model_monitor.generate_dashboard("lab_sample_classifier")
        if dashboard_path:
            print(f"✅ Generated dashboard: {dashboard_path}")

    async def demo_ab_testing(self) -> None:
        """Demonstrate A/B testing functionality"""
        print("\n🔬 === A/B TESTING DEMO ===")

        # Create A/B test configuration
        ab_config = ABTestConfig(
            test_id="",
            name="Model Version Comparison",
            description="Testing new model version against current production",
            test_type=TestType.CHAMPION_CHALLENGER,
            control_model_id="lab_sample_classifier",
            control_model_version="1.0.0",
            treatment_models=[{"model_id": "lab_sample_classifier", "version": "1.1.0"}],
            traffic_allocation={"control": 0.7, "treatment_1": 0.3},
            hypothesis="New model version will improve accuracy by 2%",
            primary_metric="accuracy",
            secondary_metrics=["precision", "recall", "latency"],
            minimum_detectable_effect=0.02,
            planned_duration_days=7,
        )

        # Create and start A/B test
        test_id = await self.ab_test_manager.create_test(ab_config)
        print(f"✅ Created A/B test: {test_id}")

        success = await self.ab_test_manager.start_test(test_id)
        if success:
            print("✅ A/B test started")

        # Simulate some test interactions
        print("Simulating test interactions...")
        for i in range(50):
            user_id = f"user_{i}"
            variant = await self.ab_test_manager.assign_variant(test_id, user_id)

            if variant:
                # Simulate prediction with slightly different performance for variants
                if variant == "control":
                    accuracy = np.random.uniform(0.85, 0.90)
                    latency = np.random.normal(200, 30)
                else:  # treatment
                    accuracy = np.random.uniform(0.87, 0.92)  # Slightly better
                    latency = np.random.normal(180, 25)  # Slightly faster

                interaction = ABTestInteraction(
                    interaction_id=f"interaction_{i}",
                    test_id=test_id,
                    variant_id=variant,
                    user_id=user_id,
                    request_data={"sample_type": "blood"},
                    response_data={"accuracy": accuracy, "prediction": "approved"},
                    latency_ms=latency,
                    success=True,
                    conversion=accuracy > 0.9,
                )

                await self.ab_test_manager.log_interaction(interaction)

        print("✅ Logged test interactions")

        # Calculate results
        results = await self.ab_test_manager.calculate_results(test_id)
        print("✅ A/B test results:")
        for variant_id, result in results.items():
            print(f"   {variant_id}: {result.accuracy:.3f} accuracy, {result.sample_size} samples")

    async def demo_continuous_learning(self) -> None:
        """Demonstrate continuous learning pipeline"""
        print("\n🔄 === CONTINUOUS LEARNING DEMO ===")

        # Register a training pipeline
        async def sample_training_pipeline(
            train_data, val_data, test_data, config, experiment_tracker
        ):
            """Sample training pipeline for continuous learning"""
            # Simulate training
            await asyncio.sleep(2)

            # Create a simple model
            model = RandomForestClassifier(n_estimators=50, random_state=42)

            # Mock training data
            X = np.random.randn(100, 5)
            y = np.random.choice([0, 1], size=100)
            model.fit(X, y)

            hyperparameters = {"n_estimators": 50, "random_state": 42}
            metrics = {"accuracy": 0.88, "precision": 0.86, "recall": 0.90, "f1_score": 0.88}

            return model, hyperparameters, metrics

        self.continuous_learning.register_training_pipeline(
            "sample_pipeline", sample_training_pipeline
        )

        # Create retraining configuration
        retrain_config = RetrainingConfig(
            config_id="",
            model_type="lab_sample_classifier",
            training_pipeline="sample_pipeline",
            min_retrain_interval_hours=1,  # Very short for demo
            new_data_threshold=10,
            minimum_accuracy=0.80,
            minimum_improvement=0.01,
        )

        config_id = await self.continuous_learning.create_retraining_config(retrain_config)
        print(f"✅ Created retraining configuration: {config_id}")

        # Trigger manual retraining
        run_id = await self.continuous_learning.trigger_retraining(
            config_id, TriggerType.MANUAL, "Demo manual retraining"
        )

        if run_id:
            print(f"✅ Triggered retraining: {run_id}")

            # Wait for training to complete
            await asyncio.sleep(5)

            training_run = await self.continuous_learning.get_training_run(run_id)
            if training_run:
                print(f"✅ Training completed: {training_run.status.value}")
                print(f"   Metrics: {training_run.metrics}")

    async def demo_deployment(self) -> None:
        """Demonstrate model deployment"""
        print("\n🚀 === DEPLOYMENT DEMO ===")

        # Start deployment monitoring
        await self.deployment_manager.start_monitoring()

        # Create deployment configuration
        deploy_config = DeploymentConfig(
            deployment_id="",
            model_id="lab_sample_classifier",
            model_version="1.0.0",
            environment=DeploymentEnvironment.STAGING,
            strategy=DeploymentStrategy.ROLLING,
            replicas=1,
            health_check_path="/health",
            auto_rollback_enabled=True,
            environment_variables={"MODEL_ENV": "staging"},
        )

        # Deploy model
        deployment_id = await self.deployment_manager.deploy_model(deploy_config)
        print(f"✅ Started deployment: {deployment_id}")

        # Wait for deployment to complete
        await asyncio.sleep(10)

        deployment = await self.deployment_manager.get_deployment_record(deployment_id)
        if deployment:
            print(f"✅ Deployment status: {deployment.status.value}")
            if deployment.endpoint_url:
                print(f"   Endpoint: {deployment.endpoint_url}")

    def _create_sample_lab_data(self) -> pd.DataFrame:
        """Create sample laboratory data for demonstration"""
        np.random.seed(42)
        n_samples = 1000

        data = {
            "sample_id": [f"S{i:06d}" for i in range(n_samples)],
            "submission_date": pd.date_range("2023-01-01", periods=n_samples, freq="H"),
            "sample_type": np.random.choice(["blood", "urine", "tissue", "saliva"], n_samples),
            "concentration": np.random.normal(50, 15, n_samples),
            "quality_score": np.random.uniform(0.3, 1.0, n_samples),
            "processing_time": np.random.exponential(30, n_samples),
            "approved": np.random.choice([True, False], n_samples, p=[0.8, 0.2]),
        }

        # Add some missing values and outliers for realism
        missing_indices = np.random.choice(n_samples, size=int(n_samples * 0.05), replace=False)
        data["concentration"][missing_indices] = np.nan

        # Add some outliers
        outlier_indices = np.random.choice(n_samples, size=int(n_samples * 0.02), replace=False)
        data["concentration"][outlier_indices] = np.random.uniform(200, 300, len(outlier_indices))

        return pd.DataFrame(data)

    async def run_complete_demo(self) -> None:
        """Run the complete MLOps pipeline demonstration"""
        print("🎯 TRACSEQ 2.0 MLOPS PIPELINE DEMONSTRATION")
        print("=" * 60)

        try:
            # Initialize components
            await self.initialize_components()

            # Run data pipeline demo
            processed_data_path = await self.demo_data_pipeline()

            if processed_data_path:
                # Run experiment tracking demo
                model, experiment_id, metrics = await self.demo_experiment_tracking(
                    processed_data_path
                )

                # Run model registry demo
                model_id = await self.demo_model_registry(model, experiment_id, metrics)

                # Run monitoring demo
                await self.demo_monitoring(model_id)

                # Run A/B testing demo
                await self.demo_ab_testing()

                # Run continuous learning demo
                await self.demo_continuous_learning()

                # Run deployment demo
                await self.demo_deployment()

            print("\n🎉 === DEMO COMPLETED SUCCESSFULLY ===")
            print("All MLOps components have been demonstrated!")
            print("\nNext steps:")
            print("1. Integrate with your actual laboratory data")
            print("2. Configure production databases and infrastructure")
            print("3. Set up monitoring and alerting")
            print("4. Implement custom training pipelines")
            print("5. Configure deployment to your target environment")

        except Exception as e:
            print(f"\n❌ Demo failed with error: {str(e)}")
            raise

        finally:
            # Cleanup
            if self.model_monitor:
                await self.model_monitor.stop_monitoring()
            if self.deployment_manager:
                await self.deployment_manager.stop_monitoring()


# Example usage
if __name__ == "__main__":
    # Alert handler example
    async def email_alert_handler(alert):
        """Example email alert handler"""
        print(f"📧 EMAIL ALERT: {alert.severity.value.upper()} - {alert.message}")

    async def slack_alert_handler(alert):
        """Example Slack alert handler"""
        print(f"💬 SLACK ALERT: {alert.severity.value.upper()} - {alert.message}")

    # Run the demonstration
    demo = TracSeqMLOpsDemo()
    asyncio.run(demo.run_complete_demo())
