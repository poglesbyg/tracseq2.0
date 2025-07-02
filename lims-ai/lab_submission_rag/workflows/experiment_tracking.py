"""
Experiment Tracking Workflow for MLOps

This workflow manages:
1. Model performance tracking
2. A/B testing between different models
3. Continuous learning pipeline
4. Model versioning and deployment decisions
"""

import asyncio
import json
import logging
import os
import pickle
import shutil
import time
from datetime import datetime
from pathlib import Path
from typing import Any, Dict, List, Optional

import numpy as np
from llama_index.workflows import (
    Context,
    Event,
    StartEvent,
    StopEvent,
    Workflow,
    step,
)
from pydantic import BaseModel, Field

from ..config import settings
from ..mlops.experiment_tracker import ExperimentTracker
from ..mlops.model_registry import ModelRegistry
from ..rag.llm_interface import LLMInterface

logger = logging.getLogger(__name__)


# Define workflow events
class ExperimentStartEvent(Event):
    """Start a new experiment"""
    experiment_name: str
    model_configs: List[Dict[str, Any]]
    test_data: List[Dict[str, Any]]
    metrics_to_track: List[str]
    baseline_model_id: Optional[str] = None


class ModelEvaluationEvent(Event):
    """Evaluate a single model"""
    model_id: str
    model_config: Dict[str, Any]
    test_data: List[Dict[str, Any]]
    experiment_id: str


class EvaluationCompleteEvent(Event):
    """Model evaluation completed"""
    model_id: str
    metrics: Dict[str, float]
    predictions: List[Any]
    processing_time: float
    model_path: Optional[str] = None


class ComparisonEvent(Event):
    """Compare model results"""
    experiment_id: str
    results: Dict[str, Dict[str, Any]]


class DeploymentDecisionEvent(Event):
    """Deployment decision based on results"""
    winning_model_id: str
    deploy: bool
    reason: str
    metrics_comparison: Dict[str, Any]
    improvement_percentage: float


class ModelDeployedEvent(Event):
    """Model deployment completed"""
    model_id: str
    deployment_status: str
    deployment_time: datetime
    model_version: str
    deployment_endpoint: Optional[str] = None


class ExperimentErrorEvent(Event):
    """Experiment encountered an error"""
    error_message: str
    step_name: str
    model_id: Optional[str] = None
    recoverable: bool


class ExperimentTrackingWorkflow(Workflow):
    """
    MLOps workflow for experiment tracking and model comparison.
    
    Manages A/B testing, performance tracking, and deployment decisions.
    """
    
    def __init__(self, timeout: int = 1800, verbose: bool = True):
        super().__init__(timeout=timeout, verbose=verbose)
        self.experiment_tracker = ExperimentTracker()
        self.model_registry = ModelRegistry()
        self.models_dir = Path(getattr(settings, "models_directory", "./models"))
        self.models_dir.mkdir(exist_ok=True)
        self.llm_interface = LLMInterface()
        
    @step(pass_context=True)
    async def initialize_experiment(
        self, ctx: Context, ev: StartEvent
    ) -> List[ModelEvaluationEvent]:
        """Initialize experiment and create evaluation events for each model"""
        
        experiment_name = getattr(ev, "experiment_name", None)
        model_configs = getattr(ev, "model_configs", [])
        test_data = getattr(ev, "test_data", [])
        metrics_to_track = getattr(ev, "metrics_to_track", ["accuracy", "f1_score", "processing_time"])
        baseline_model_id = getattr(ev, "baseline_model_id", None)
        
        # Create experiment
        experiment_id = await self.experiment_tracker.create_experiment(
            name=experiment_name,
            description=f"A/B test with {len(model_configs)} models",
            tags=["workflow", "ab_test", "laboratory"],
            metadata={
                "model_count": len(model_configs),
                "test_data_size": len(test_data),
                "baseline_model": baseline_model_id
            }
        )
        
        # Store in context
        ctx.data["experiment_id"] = experiment_id
        ctx.data["metrics_to_track"] = metrics_to_track
        ctx.data["model_results"] = {}
        ctx.data["start_time"] = time.time()
        ctx.data["model_configs"] = model_configs
        ctx.data["baseline_model_id"] = baseline_model_id
        ctx.data["cleanup_paths"] = []  # Track paths to clean up
        
        logger.info(f"Started experiment {experiment_id} with {len(model_configs)} models")
        
        # Create evaluation events for parallel processing
        evaluation_events = []
        for i, config in enumerate(model_configs):
            model_id = config.get("model_id", f"model_{i}")
            evaluation_events.append(
                ModelEvaluationEvent(
                    model_id=model_id,
                    model_config=config,
                    test_data=test_data,
                    experiment_id=experiment_id
                )
            )
        
        return evaluation_events
    
    @step(pass_context=True)
    async def evaluate_model(
        self, ctx: Context, ev: ModelEvaluationEvent
    ) -> EvaluationCompleteEvent | ExperimentErrorEvent:
        """Evaluate a single model on test data"""
        
        logger.info(f"Evaluating model {ev.model_id}")
        start_time = time.time()
        
        try:
            # Load or create model based on config
            model, model_path = await self._load_or_create_model(ev.model_config, ev.model_id)
            
            # Track for cleanup
            if model_path:
                ctx.data["cleanup_paths"].append(model_path)
            
            # Run predictions
            predictions = []
            metrics = {
                "total_samples": len(ev.test_data),
                "successful_predictions": 0,
                "failed_predictions": 0,
                "confidence_scores": [],
                "processing_times": []
            }
            
            for test_sample in ev.test_data:
                sample_start = time.time()
                try:
                    # Run model prediction
                    prediction = await self._run_model_prediction(
                        model, 
                        test_sample,
                        ev.model_config
                    )
                    predictions.append(prediction)
                    
                    # Track metrics
                    metrics["successful_predictions"] += 1
                    if "confidence" in prediction:
                        metrics["confidence_scores"].append(prediction["confidence"])
                    
                    # Track processing time per sample
                    sample_time = time.time() - sample_start
                    metrics["processing_times"].append(sample_time)
                    
                    # Calculate accuracy if ground truth is available
                    if "expected" in test_sample and "prediction" in prediction:
                        if test_sample["expected"] == prediction["prediction"]:
                            metrics.setdefault("correct_predictions", 0)
                            metrics["correct_predictions"] += 1
                        
                except Exception as e:
                    logger.error(f"Prediction failed for sample: {e}")
                    metrics["failed_predictions"] += 1
                    predictions.append({"error": str(e), "sample_id": test_sample.get("id")})
            
            # Calculate aggregate metrics
            if metrics["confidence_scores"]:
                metrics["avg_confidence"] = float(np.mean(metrics["confidence_scores"]))
                metrics["min_confidence"] = float(np.min(metrics["confidence_scores"]))
                metrics["max_confidence"] = float(np.max(metrics["confidence_scores"]))
                metrics["confidence_std"] = float(np.std(metrics["confidence_scores"]))
            
            if metrics["processing_times"]:
                metrics["avg_processing_time"] = float(np.mean(metrics["processing_times"]))
                metrics["p95_processing_time"] = float(np.percentile(metrics["processing_times"], 95))
            
            # Calculate accuracy
            if "correct_predictions" in metrics:
                metrics["accuracy"] = metrics["correct_predictions"] / metrics["total_samples"]
            
            metrics["success_rate"] = metrics["successful_predictions"] / metrics["total_samples"]
            metrics["processing_time"] = time.time() - start_time
            
            # Calculate F1 score if we have binary classification results
            if all("prediction" in p and isinstance(p.get("prediction"), bool) for p in predictions[:10]):
                metrics["f1_score"] = await self._calculate_f1_score(predictions, ev.test_data)
            
            # Log to experiment tracker
            await self.experiment_tracker.log_metrics(
                experiment_id=ev.experiment_id,
                run_id=ev.model_id,
                metrics=metrics,
                step=0
            )
            
            # Log model artifacts
            if model_path:
                await self.experiment_tracker.log_artifact(
                    experiment_id=ev.experiment_id,
                    run_id=ev.model_id,
                    artifact_path=model_path,
                    artifact_type="model"
                )
            
            return EvaluationCompleteEvent(
                model_id=ev.model_id,
                metrics=metrics,
                predictions=predictions,
                processing_time=metrics["processing_time"],
                model_path=model_path
            )
            
        except Exception as e:
            logger.error(f"Model evaluation failed: {e}")
            return ExperimentErrorEvent(
                error_message=str(e),
                step_name="evaluate_model",
                model_id=ev.model_id,
                recoverable=False
            )
    
    @step(pass_context=True)
    async def handle_experiment_error(
        self, ctx: Context, ev: ExperimentErrorEvent
    ) -> StopEvent:
        """Handle experiment errors and cleanup"""
        
        logger.error(f"Experiment error in {ev.step_name}: {ev.error_message}")
        
        # Log error to experiment tracker
        await self.experiment_tracker.log_error(
            experiment_id=ctx.data["experiment_id"],
            error_message=ev.error_message,
            step=ev.step_name,
            model_id=ev.model_id
        )
        
        # Cleanup any created files
        await self._cleanup_experiment_files(ctx)
        
        # Mark experiment as failed
        await self.experiment_tracker.close_experiment(
            experiment_id=ctx.data["experiment_id"],
            status="failed",
            summary={
                "error": ev.error_message,
                "failed_step": ev.step_name,
                "failed_model": ev.model_id
            }
        )
        
        return StopEvent(
            result={
                "experiment_id": ctx.data["experiment_id"],
                "status": "failed",
                "error": ev.error_message,
                "total_time": time.time() - ctx.data["start_time"]
            }
        )
    
    @step(pass_context=True)
    async def collect_results(
        self, ctx: Context, ev: EvaluationCompleteEvent
    ) -> ComparisonEvent | None:
        """Collect evaluation results from all models"""
        
        # Store model results
        ctx.data["model_results"][ev.model_id] = {
            "metrics": ev.metrics,
            "predictions": ev.predictions,
            "processing_time": ev.processing_time,
            "model_path": ev.model_path
        }
        
        # Check if all models have been evaluated
        total_models = len(ctx.data.get("model_configs", []))
        completed_models = len(ctx.data["model_results"])
        
        logger.info(f"Collected results for {ev.model_id} ({completed_models}/{total_models})")
        
        if completed_models >= total_models:
            # All models evaluated, trigger comparison
            return ComparisonEvent(
                experiment_id=ctx.data["experiment_id"],
                results=ctx.data["model_results"]
            )
        
        # Still waiting for other models
        return None
    
    @step(pass_context=True)
    async def compare_models(
        self, ctx: Context, ev: ComparisonEvent
    ) -> DeploymentDecisionEvent:
        """Compare model results and make deployment decision"""
        
        logger.info("Comparing model results")
        
        # Calculate comparison metrics
        comparison = {}
        best_model_id = None
        best_score = -1
        
        for model_id, results in ev.results.items():
            metrics = results["metrics"]
            
            # Calculate composite score
            if "error" not in metrics:
                # Weighted scoring based on different metrics
                accuracy_weight = 0.4
                confidence_weight = 0.2
                speed_weight = 0.2
                success_weight = 0.2
                
                accuracy_score = metrics.get("accuracy", metrics.get("success_rate", 0))
                confidence_score = metrics.get("avg_confidence", 0)
                # Inverse of processing time normalized
                speed_score = 1.0 / (1.0 + metrics.get("avg_processing_time", 1.0))
                success_score = metrics.get("success_rate", 0)
                
                composite_score = (
                    accuracy_score * accuracy_weight +
                    confidence_score * confidence_weight +
                    speed_score * speed_weight +
                    success_score * success_weight
                )
                
                comparison[model_id] = {
                    "composite_score": composite_score,
                    "accuracy": accuracy_score,
                    "avg_confidence": confidence_score,
                    "avg_processing_time": metrics.get("avg_processing_time", 0),
                    "success_rate": success_score,
                    "total_samples": metrics.get("total_samples", 0)
                }
                
                if composite_score > best_score:
                    best_score = composite_score
                    best_model_id = model_id
            else:
                comparison[model_id] = {
                    "composite_score": 0,
                    "error": metrics["error"]
                }
        
        # Make deployment decision
        deploy = False
        reason = "No suitable model found"
        improvement_percentage = 0.0
        
        if best_model_id and best_score > 0.7:  # Deployment threshold
            deploy = True
            reason = f"Model {best_model_id} achieved score of {best_score:.3f}"
            
            # Compare with baseline if available
            baseline_id = ctx.data.get("baseline_model_id")
            if baseline_id and baseline_id in comparison:
                baseline_score = comparison[baseline_id]["composite_score"]
                improvement_percentage = ((best_score - baseline_score) / baseline_score) * 100
                
                # Require significant improvement over baseline
                if improvement_percentage < 5:  # 5% improvement threshold
                    deploy = False
                    reason = f"Improvement over baseline insufficient ({improvement_percentage:.1f}%)"
                else:
                    reason = f"Model {best_model_id} shows {improvement_percentage:.1f}% improvement over baseline"
            
            # Check against current production model
            current_prod_score = await self._get_current_production_score()
            if current_prod_score:
                prod_improvement = ((best_score - current_prod_score) / current_prod_score) * 100
                if prod_improvement < 5:
                    deploy = False
                    reason = f"New model not significantly better than production ({prod_improvement:.1f}% improvement)"
        
        # Log comparison results
        await self.experiment_tracker.log_comparison(
            experiment_id=ev.experiment_id,
            comparison_data=comparison,
            winner=best_model_id,
            decision={
                "deploy": deploy, 
                "reason": reason,
                "improvement_percentage": improvement_percentage
            }
        )
        
        return DeploymentDecisionEvent(
            winning_model_id=best_model_id or "none",
            deploy=deploy,
            reason=reason,
            metrics_comparison=comparison,
            improvement_percentage=improvement_percentage
        )
    
    @step(pass_context=True)
    async def handle_deployment(
        self, ctx: Context, ev: DeploymentDecisionEvent
    ) -> ModelDeployedEvent | StopEvent:
        """Handle model deployment based on decision"""
        
        if not ev.deploy:
            logger.info(f"Deployment skipped: {ev.reason}")
            
            # Cleanup experiment files
            await self._cleanup_experiment_files(ctx)
            
            return StopEvent(
                result={
                    "experiment_id": ctx.data["experiment_id"],
                    "deployed": False,
                    "reason": ev.reason,
                    "comparison": ev.metrics_comparison,
                    "total_time": time.time() - ctx.data["start_time"]
                }
            )
        
        logger.info(f"Deploying model {ev.winning_model_id}")
        
        try:
            # Get model path
            model_path = ctx.data["model_results"][ev.winning_model_id].get("model_path")
            
            # Register model in model registry
            model_version = await self.model_registry.register_model(
                model_name=f"lab_extraction_{ev.winning_model_id}",
                model_path=model_path,
                metrics=ev.metrics_comparison[ev.winning_model_id],
                tags=["production", "workflow_deployed"],
                metadata={
                    "experiment_id": ctx.data["experiment_id"],
                    "improvement_percentage": ev.improvement_percentage,
                    "deployment_reason": ev.reason
                }
            )
            
            # Deploy model (simulate actual deployment)
            deployment_endpoint = await self._deploy_model(
                model_id=ev.winning_model_id,
                model_version=model_version,
                model_path=model_path
            )
            
            deployment_status = "success"
            deployment_time = datetime.utcnow()
            
            # Update experiment with deployment info
            await self.experiment_tracker.log_deployment(
                experiment_id=ctx.data["experiment_id"],
                model_id=ev.winning_model_id,
                model_version=model_version,
                deployment_time=deployment_time,
                status=deployment_status,
                endpoint=deployment_endpoint
            )
            
            # Don't cleanup the deployed model
            if model_path in ctx.data.get("cleanup_paths", []):
                ctx.data["cleanup_paths"].remove(model_path)
            
            return ModelDeployedEvent(
                model_id=ev.winning_model_id,
                deployment_status=deployment_status,
                deployment_time=deployment_time,
                model_version=model_version,
                deployment_endpoint=deployment_endpoint
            )
            
        except Exception as e:
            logger.error(f"Deployment failed: {e}")
            
            # Cleanup on failure
            await self._cleanup_experiment_files(ctx)
            
            return StopEvent(
                result={
                    "experiment_id": ctx.data["experiment_id"],
                    "deployed": False,
                    "reason": f"Deployment failed: {str(e)}",
                    "comparison": ev.metrics_comparison,
                    "total_time": time.time() - ctx.data["start_time"]
                }
            )
    
    @step(pass_context=True)
    async def finalize_experiment(
        self, ctx: Context, ev: ModelDeployedEvent
    ) -> StopEvent:
        """Finalize experiment and return results"""
        
        total_time = time.time() - ctx.data["start_time"]
        
        # Cleanup non-deployed model files
        await self._cleanup_experiment_files(ctx, exclude_deployed=True)
        
        result = {
            "experiment_id": ctx.data["experiment_id"],
            "deployed": True,
            "deployed_model": ev.model_id,
            "deployment_time": ev.deployment_time.isoformat(),
            "deployment_status": ev.deployment_status,
            "deployment_endpoint": ev.deployment_endpoint,
            "model_version": ev.model_version,
            "total_time": total_time,
            "models_evaluated": len(ctx.data["model_results"]),
            "comparison": ctx.data.get("comparison", {})
        }
        
        # Close experiment
        await self.experiment_tracker.close_experiment(
            experiment_id=ctx.data["experiment_id"],
            status="completed",
            summary=result
        )
        
        logger.info(f"Experiment completed in {total_time:.2f}s")
        
        return StopEvent(result=result)
    
    async def _load_or_create_model(
        self, model_config: Dict[str, Any], model_id: str
    ) -> tuple[Any, str]:
        """Load or create model based on configuration"""
        
        model_type = model_config.get("type", "llm_extractor")
        model_path = self.models_dir / f"{model_id}.pkl"
        
        if model_type == "llm_extractor":
            # Create LLM-based extractor model
            model = {
                "type": "llm_extractor",
                "config": model_config,
                "interface": self.llm_interface,
                "created_at": datetime.utcnow().isoformat()
            }
        elif model_type == "rule_based":
            # Create rule-based extractor
            model = {
                "type": "rule_based",
                "rules": model_config.get("rules", {}),
                "created_at": datetime.utcnow().isoformat()
            }
        else:
            # Load existing model if specified
            existing_path = model_config.get("model_path")
            if existing_path and Path(existing_path).exists():
                with open(existing_path, 'rb') as f:
                    model = pickle.load(f)
            else:
                # Default model
                model = {
                    "type": "default",
                    "config": model_config,
                    "created_at": datetime.utcnow().isoformat()
                }
        
        # Save model
        with open(model_path, 'wb') as f:
            pickle.dump(model, f)
        
        return model, str(model_path)
    
    async def _run_model_prediction(
        self, model: Any, test_sample: Dict[str, Any], model_config: Dict[str, Any]
    ) -> Dict[str, Any]:
        """Run model prediction on a test sample"""
        
        model_type = model.get("type", "default")
        
        if model_type == "llm_extractor":
            # Use LLM for extraction
            text = test_sample.get("text", "")
            if not text:
                return {"error": "No text in sample", "sample_id": test_sample.get("id")}
            
            # Create extraction prompt based on config
            prompt = model_config.get("prompt_template", "Extract information from: {text}")
            prompt = prompt.format(text=text)
            
            # Get LLM response
            try:
                result = await model["interface"].extract_submission_info_with_prompt(prompt)
                
                # Calculate confidence based on response
                confidence = 0.5  # Base confidence
                if isinstance(result, dict) and not result.get("error"):
                    confidence = 0.8
                    if all(k in result for k in ["administrative", "sample", "sequencing"]):
                        confidence = 0.95
                
                return {
                    "prediction": result,
                    "confidence": confidence,
                    "sample_id": test_sample.get("id"),
                    "model_type": model_type
                }
            except Exception as e:
                return {
                    "error": str(e),
                    "sample_id": test_sample.get("id"),
                    "model_type": model_type
                }
                
        elif model_type == "rule_based":
            # Apply rules for extraction
            text = test_sample.get("text", "")
            rules = model.get("rules", {})
            
            extracted = {}
            confidence = 0.0
            matches = 0
            
            for field, pattern in rules.items():
                import re
                match = re.search(pattern, text, re.IGNORECASE)
                if match:
                    extracted[field] = match.group(1) if match.groups() else match.group(0)
                    matches += 1
            
            if matches > 0:
                confidence = matches / len(rules)
            
            return {
                "prediction": extracted,
                "confidence": confidence,
                "sample_id": test_sample.get("id"),
                "model_type": model_type
            }
        
        else:
            # Default prediction
            return {
                "prediction": {"status": "processed"},
                "confidence": 0.5,
                "sample_id": test_sample.get("id"),
                "model_type": model_type
            }
    
    async def _calculate_f1_score(
        self, predictions: List[Dict[str, Any]], test_data: List[Dict[str, Any]]
    ) -> float:
        """Calculate F1 score for binary classification"""
        
        true_positives = 0
        false_positives = 0
        false_negatives = 0
        
        for pred, test in zip(predictions, test_data):
            if "error" in pred:
                continue
                
            predicted = pred.get("prediction", {}).get("has_quality_issues", False)
            actual = test.get("expected", {}).get("has_quality_issues", False)
            
            if predicted and actual:
                true_positives += 1
            elif predicted and not actual:
                false_positives += 1
            elif not predicted and actual:
                false_negatives += 1
        
        if true_positives == 0:
            return 0.0
        
        precision = true_positives / (true_positives + false_positives) if (true_positives + false_positives) > 0 else 0
        recall = true_positives / (true_positives + false_negatives) if (true_positives + false_negatives) > 0 else 0
        
        if precision + recall == 0:
            return 0.0
        
        f1 = 2 * (precision * recall) / (precision + recall)
        return f1
    
    async def _get_current_production_score(self) -> Optional[float]:
        """Get performance score of current production model"""
        
        try:
            # Query model registry for current production model
            production_model = await self.model_registry.get_production_model("lab_extraction")
            
            if production_model and "metrics" in production_model:
                return production_model["metrics"].get("composite_score", 0.75)
        except:
            pass
        
        # Default baseline score
        return 0.75
    
    async def _deploy_model(
        self, model_id: str, model_version: str, model_path: Optional[str]
    ) -> str:
        """Deploy model to production"""
        
        # In a real implementation, this would:
        # 1. Package the model
        # 2. Deploy to serving infrastructure (e.g., Kubernetes, SageMaker)
        # 3. Update load balancer/routing
        # 4. Run health checks
        
        # For now, simulate deployment
        deployment_dir = self.models_dir / "deployed"
        deployment_dir.mkdir(exist_ok=True)
        
        if model_path and Path(model_path).exists():
            # Copy model to deployment directory
            deployed_path = deployment_dir / f"{model_id}_{model_version}.pkl"
            shutil.copy2(model_path, deployed_path)
            
            # Simulate endpoint creation
            endpoint = f"http://model-serving.lab.local/v1/models/{model_id}/versions/{model_version}"
            
            logger.info(f"Model deployed to: {endpoint}")
            return endpoint
        
        return f"http://model-serving.lab.local/v1/models/{model_id}/versions/{model_version}"
    
    async def _cleanup_experiment_files(self, ctx: Context, exclude_deployed: bool = False):
        """Clean up temporary files created during experiment"""
        
        cleanup_paths = ctx.data.get("cleanup_paths", [])
        
        for path in cleanup_paths:
            try:
                if Path(path).exists():
                    # Skip if deployed model and exclude_deployed is True
                    if exclude_deployed and "deployed" in str(path):
                        continue
                    
                    os.remove(path)
                    logger.debug(f"Cleaned up: {path}")
            except Exception as e:
                logger.warning(f"Failed to cleanup {path}: {e}")


# Example usage function
async def run_mlops_experiment(
    experiment_name: str,
    model_configs: List[Dict[str, Any]],
    test_data: List[Dict[str, Any]],
    baseline_model_id: Optional[str] = None
) -> Dict[str, Any]:
    """Run an MLOps experiment with multiple models
    
    Args:
        experiment_name: Name of the experiment
        model_configs: List of model configurations to test
        test_data: Test dataset for evaluation
        baseline_model_id: Optional ID of baseline model to compare against
    
    Returns:
        Experiment results including deployment status
    """
    
    workflow = ExperimentTrackingWorkflow()
    
    result = await workflow.run(
        experiment_name=experiment_name,
        model_configs=model_configs,
        test_data=test_data,
        metrics_to_track=["accuracy", "f1_score", "processing_time", "confidence"],
        baseline_model_id=baseline_model_id
    )
    
    return result