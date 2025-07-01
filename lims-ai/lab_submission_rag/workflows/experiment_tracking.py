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
import time
from datetime import datetime
from typing import Any, Dict, List, Optional

from llama_index.workflows import (
    Context,
    Event,
    StartEvent,
    StopEvent,
    Workflow,
    step,
)
from pydantic import BaseModel, Field

from ..mlops.experiment_tracker import ExperimentTracker
from ..mlops.model_registry import ModelRegistry

logger = logging.getLogger(__name__)


# Define workflow events
class ExperimentStartEvent(Event):
    """Start a new experiment"""
    experiment_name: str
    model_configs: List[Dict[str, Any]]
    test_data: List[Dict[str, Any]]
    metrics_to_track: List[str]


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


class ModelDeployedEvent(Event):
    """Model deployment completed"""
    model_id: str
    deployment_status: str
    deployment_time: datetime


class ExperimentTrackingWorkflow(Workflow):
    """
    MLOps workflow for experiment tracking and model comparison.
    
    Manages A/B testing, performance tracking, and deployment decisions.
    """
    
    def __init__(self, timeout: int = 1800, verbose: bool = True):
        super().__init__(timeout=timeout, verbose=verbose)
        self.experiment_tracker = ExperimentTracker()
        self.model_registry = ModelRegistry()
        
    @step(pass_context=True)
    async def initialize_experiment(
        self, ctx: Context, ev: StartEvent
    ) -> List[ModelEvaluationEvent]:
        """Initialize experiment and create evaluation events for each model"""
        
        experiment_name = ev.get("experiment_name")
        model_configs = ev.get("model_configs")
        test_data = ev.get("test_data")
        metrics_to_track = ev.get("metrics_to_track", ["accuracy", "f1_score", "processing_time"])
        
        # Create experiment
        experiment_id = await self.experiment_tracker.create_experiment(
            name=experiment_name,
            description=f"A/B test with {len(model_configs)} models",
            tags=["workflow", "ab_test", "laboratory"]
        )
        
        # Store in context
        ctx.data["experiment_id"] = experiment_id
        ctx.data["metrics_to_track"] = metrics_to_track
        ctx.data["model_results"] = {}
        ctx.data["start_time"] = time.time()
        
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
    ) -> EvaluationCompleteEvent:
        """Evaluate a single model on test data"""
        
        logger.info(f"Evaluating model {ev.model_id}")
        start_time = time.time()
        
        try:
            # Load or create model based on config
            model = await self._load_or_create_model(ev.model_config)
            
            # Run predictions
            predictions = []
            metrics = {
                "total_samples": len(ev.test_data),
                "successful_predictions": 0,
                "failed_predictions": 0,
                "confidence_scores": []
            }
            
            for test_sample in ev.test_data:
                try:
                    # Simulate model prediction (replace with actual model inference)
                    prediction = await self._run_model_prediction(
                        model, 
                        test_sample
                    )
                    predictions.append(prediction)
                    
                    # Track metrics
                    metrics["successful_predictions"] += 1
                    if "confidence" in prediction:
                        metrics["confidence_scores"].append(prediction["confidence"])
                        
                except Exception as e:
                    logger.error(f"Prediction failed for sample: {e}")
                    metrics["failed_predictions"] += 1
                    predictions.append({"error": str(e)})
            
            # Calculate aggregate metrics
            if metrics["confidence_scores"]:
                metrics["avg_confidence"] = sum(metrics["confidence_scores"]) / len(metrics["confidence_scores"])
                metrics["min_confidence"] = min(metrics["confidence_scores"])
                metrics["max_confidence"] = max(metrics["confidence_scores"])
            
            metrics["success_rate"] = metrics["successful_predictions"] / metrics["total_samples"]
            metrics["processing_time"] = time.time() - start_time
            
            # Log to experiment tracker
            await self.experiment_tracker.log_metrics(
                experiment_id=ev.experiment_id,
                run_id=ev.model_id,
                metrics=metrics,
                step=0
            )
            
            return EvaluationCompleteEvent(
                model_id=ev.model_id,
                metrics=metrics,
                predictions=predictions,
                processing_time=metrics["processing_time"]
            )
            
        except Exception as e:
            logger.error(f"Model evaluation failed: {e}")
            return EvaluationCompleteEvent(
                model_id=ev.model_id,
                metrics={"error": str(e), "processing_time": time.time() - start_time},
                predictions=[],
                processing_time=time.time() - start_time
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
            "processing_time": ev.processing_time
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
            
            # Calculate composite score (customize based on your needs)
            if "error" not in metrics:
                score = (
                    metrics.get("success_rate", 0) * 0.4 +
                    metrics.get("avg_confidence", 0) * 0.3 +
                    (1.0 / (1.0 + metrics.get("processing_time", 999))) * 0.3
                )
                
                comparison[model_id] = {
                    "score": score,
                    "success_rate": metrics.get("success_rate", 0),
                    "avg_confidence": metrics.get("avg_confidence", 0),
                    "processing_time": metrics.get("processing_time", 0)
                }
                
                if score > best_score:
                    best_score = score
                    best_model_id = model_id
            else:
                comparison[model_id] = {
                    "score": 0,
                    "error": metrics["error"]
                }
        
        # Make deployment decision
        deploy = False
        reason = "No suitable model found"
        
        if best_model_id and best_score > 0.7:  # Deployment threshold
            deploy = True
            reason = f"Model {best_model_id} achieved score of {best_score:.3f}"
            
            # Check if significantly better than current production model
            current_prod_score = await self._get_current_production_score()
            if current_prod_score and best_score <= current_prod_score * 1.1:
                deploy = False
                reason = f"New model not significantly better than current ({best_score:.3f} vs {current_prod_score:.3f})"
        
        # Log comparison results
        await self.experiment_tracker.log_comparison(
            experiment_id=ev.experiment_id,
            comparison_data=comparison,
            winner=best_model_id,
            decision={"deploy": deploy, "reason": reason}
        )
        
        return DeploymentDecisionEvent(
            winning_model_id=best_model_id or "none",
            deploy=deploy,
            reason=reason,
            metrics_comparison=comparison
        )
    
    @step(pass_context=True)
    async def handle_deployment(
        self, ctx: Context, ev: DeploymentDecisionEvent
    ) -> ModelDeployedEvent | StopEvent:
        """Handle model deployment based on decision"""
        
        if not ev.deploy:
            logger.info(f"Deployment skipped: {ev.reason}")
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
            # Register model in model registry
            model_version = await self.model_registry.register_model(
                model_name=f"lab_extraction_{ev.winning_model_id}",
                model_path=f"models/{ev.winning_model_id}",
                metrics=ev.metrics_comparison[ev.winning_model_id],
                tags=["production", "workflow_deployed"]
            )
            
            # Simulate deployment (replace with actual deployment logic)
            deployment_status = "success"
            deployment_time = datetime.utcnow()
            
            # Update experiment with deployment info
            await self.experiment_tracker.log_deployment(
                experiment_id=ctx.data["experiment_id"],
                model_id=ev.winning_model_id,
                model_version=model_version,
                deployment_time=deployment_time,
                status=deployment_status
            )
            
            return ModelDeployedEvent(
                model_id=ev.winning_model_id,
                deployment_status=deployment_status,
                deployment_time=deployment_time
            )
            
        except Exception as e:
            logger.error(f"Deployment failed: {e}")
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
        
        result = {
            "experiment_id": ctx.data["experiment_id"],
            "deployed": True,
            "deployed_model": ev.model_id,
            "deployment_time": ev.deployment_time.isoformat(),
            "deployment_status": ev.deployment_status,
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
    
    async def _load_or_create_model(self, model_config: Dict[str, Any]) -> Any:
        """Load or create model based on configuration"""
        # Simulate model loading - replace with actual implementation
        return {"config": model_config, "loaded": True}
    
    async def _run_model_prediction(
        self, model: Any, test_sample: Dict[str, Any]
    ) -> Dict[str, Any]:
        """Run model prediction on a test sample"""
        # Simulate prediction - replace with actual model inference
        import random
        
        return {
            "prediction": "sample_processed",
            "confidence": random.uniform(0.7, 0.95),
            "sample_id": test_sample.get("id", "unknown")
        }
    
    async def _get_current_production_score(self) -> Optional[float]:
        """Get performance score of current production model"""
        # Simulate fetching current model performance
        return 0.75  # Example baseline score


# Example usage function
async def run_mlops_experiment(
    experiment_name: str,
    model_configs: List[Dict[str, Any]],
    test_data: List[Dict[str, Any]]
) -> Dict[str, Any]:
    """Run an MLOps experiment with multiple models"""
    
    workflow = ExperimentTrackingWorkflow()
    
    result = await workflow.run(
        experiment_name=experiment_name,
        model_configs=model_configs,
        test_data=test_data,
        metrics_to_track=["accuracy", "f1_score", "processing_time", "confidence"]
    )
    
    return result