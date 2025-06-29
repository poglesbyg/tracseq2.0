# TracSeq 2.0 - AutoML Framework
# Automated machine learning for laboratory predictions

import asyncio
import json
import logging
import os
from abc import ABC, abstractmethod
from dataclasses import dataclass
from datetime import datetime, timedelta
from enum import Enum
from typing import Any, Dict, List, Optional, Tuple, Union
from uuid import UUID, uuid4

import numpy as np
import pandas as pd
from sklearn.model_selection import train_test_split, cross_val_score, GridSearchCV, RandomizedSearchCV
from sklearn.preprocessing import StandardScaler, LabelEncoder
from sklearn.metrics import accuracy_score, precision_score, recall_score, f1_score, mean_squared_error, r2_score
from sklearn.ensemble import RandomForestClassifier, RandomForestRegressor, GradientBoostingClassifier, GradientBoostingRegressor
from sklearn.linear_model import LogisticRegression, LinearRegression
from sklearn.svm import SVC, SVR
from sklearn.neural_network import MLPClassifier, MLPRegressor
import xgboost as xgb
import lightgbm as lgb
import optuna
from optuna.samplers import TPESampler
import mlflow
import mlflow.sklearn
import joblib
from fastapi import FastAPI, HTTPException, BackgroundTasks
from pydantic import BaseModel, Field
from sqlalchemy import create_engine, Column, String, Float, Integer, DateTime, JSON, Boolean
from sqlalchemy.ext.declarative import declarative_base
from sqlalchemy.orm import sessionmaker

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Database Models
Base = declarative_base()

class ProblemType(str, Enum):
    CLASSIFICATION = "classification"
    REGRESSION = "regression"
    TIME_SERIES = "time_series"

class ModelFramework(str, Enum):
    SKLEARN = "sklearn"
    XGBOOST = "xgboost"
    LIGHTGBM = "lightgbm"
    NEURAL_NET = "neural_net"

class OptimizationMetric(str, Enum):
    # Classification metrics
    ACCURACY = "accuracy"
    PRECISION = "precision"
    RECALL = "recall"
    F1_SCORE = "f1_score"
    AUC_ROC = "auc_roc"
    
    # Regression metrics
    MSE = "mse"
    RMSE = "rmse"
    MAE = "mae"
    R2 = "r2"

# Database Models
class AutoMLExperiment(Base):
    __tablename__ = "automl_experiments"
    
    id = Column(String, primary_key=True)
    name = Column(String, nullable=False)
    problem_type = Column(String, nullable=False)
    target_column = Column(String, nullable=False)
    optimization_metric = Column(String, nullable=False)
    feature_columns = Column(JSON)
    dataset_info = Column(JSON)
    best_model_id = Column(String)
    best_score = Column(Float)
    status = Column(String)  # running, completed, failed
    created_at = Column(DateTime, default=datetime.utcnow)
    completed_at = Column(DateTime)
    metadata = Column(JSON)

class ModelCandidate(Base):
    __tablename__ = "model_candidates"
    
    id = Column(String, primary_key=True)
    experiment_id = Column(String, nullable=False)
    model_type = Column(String, nullable=False)
    framework = Column(String, nullable=False)
    hyperparameters = Column(JSON)
    cross_val_score = Column(Float)
    test_score = Column(Float)
    training_time_seconds = Column(Float)
    model_path = Column(String)
    metrics = Column(JSON)
    created_at = Column(DateTime, default=datetime.utcnow)

# Request/Response Models
class AutoMLRequest(BaseModel):
    experiment_name: str
    dataset_path: Optional[str] = None
    dataset: Optional[Dict[str, List[Any]]] = None
    target_column: str
    feature_columns: Optional[List[str]] = None
    problem_type: Optional[ProblemType] = None
    optimization_metric: Optional[OptimizationMetric] = None
    time_budget_minutes: int = Field(default=30, ge=5, le=480)
    n_trials: int = Field(default=100, ge=10, le=1000)
    test_size: float = Field(default=0.2, ge=0.1, le=0.5)
    random_state: int = Field(default=42)
    metadata: Dict[str, Any] = {}

class AutoMLResponse(BaseModel):
    experiment_id: str
    status: str
    message: str

class ExperimentStatusResponse(BaseModel):
    experiment_id: str
    name: str
    status: str
    problem_type: str
    optimization_metric: str
    best_model_id: Optional[str]
    best_score: Optional[float]
    n_models_evaluated: int
    elapsed_time_minutes: Optional[float]
    created_at: datetime
    completed_at: Optional[datetime]

# Model Selection Strategy
class ModelSelector:
    """Selects appropriate models based on problem type and data characteristics"""
    
    @staticmethod
    def get_models_for_problem(problem_type: ProblemType, n_samples: int, n_features: int) -> List[Tuple[str, Any]]:
        """Get list of models suitable for the problem"""
        models = []
        
        if problem_type == ProblemType.CLASSIFICATION:
            # Logistic Regression
            models.append(("logistic_regression", LogisticRegression(random_state=42, max_iter=1000)))
            
            # Random Forest
            models.append(("random_forest", RandomForestClassifier(random_state=42)))
            
            # Gradient Boosting
            models.append(("gradient_boosting", GradientBoostingClassifier(random_state=42)))
            
            # XGBoost
            models.append(("xgboost", xgb.XGBClassifier(random_state=42, use_label_encoder=False, eval_metric='logloss')))
            
            # LightGBM
            models.append(("lightgbm", lgb.LGBMClassifier(random_state=42, verbosity=-1)))
            
            # Neural Network (if enough data)
            if n_samples > 1000:
                models.append(("neural_network", MLPClassifier(random_state=42, max_iter=1000)))
            
        elif problem_type == ProblemType.REGRESSION:
            # Linear Regression
            models.append(("linear_regression", LinearRegression()))
            
            # Random Forest
            models.append(("random_forest", RandomForestRegressor(random_state=42)))
            
            # Gradient Boosting
            models.append(("gradient_boosting", GradientBoostingRegressor(random_state=42)))
            
            # XGBoost
            models.append(("xgboost", xgb.XGBRegressor(random_state=42)))
            
            # LightGBM
            models.append(("lightgbm", lgb.LGBMRegressor(random_state=42, verbosity=-1)))
            
            # Neural Network (if enough data)
            if n_samples > 1000:
                models.append(("neural_network", MLPRegressor(random_state=42, max_iter=1000)))
        
        return models
    
    @staticmethod
    def get_hyperparameter_space(model_name: str, problem_type: ProblemType) -> Dict[str, Any]:
        """Get hyperparameter search space for a model"""
        spaces = {
            "logistic_regression": {
                "C": [0.001, 0.01, 0.1, 1, 10, 100],
                "penalty": ["l1", "l2"],
                "solver": ["liblinear", "saga"]
            },
            "random_forest": {
                "n_estimators": [50, 100, 200, 300],
                "max_depth": [None, 10, 20, 30, 40],
                "min_samples_split": [2, 5, 10],
                "min_samples_leaf": [1, 2, 4],
                "max_features": ["sqrt", "log2", None]
            },
            "gradient_boosting": {
                "n_estimators": [50, 100, 200],
                "learning_rate": [0.01, 0.1, 0.3],
                "max_depth": [3, 5, 7, 9],
                "min_samples_split": [2, 5, 10],
                "min_samples_leaf": [1, 2, 4]
            },
            "xgboost": {
                "n_estimators": [50, 100, 200],
                "learning_rate": [0.01, 0.1, 0.3],
                "max_depth": [3, 5, 7, 9],
                "min_child_weight": [1, 3, 5],
                "gamma": [0, 0.1, 0.2],
                "subsample": [0.6, 0.8, 1.0],
                "colsample_bytree": [0.6, 0.8, 1.0]
            },
            "lightgbm": {
                "n_estimators": [50, 100, 200],
                "learning_rate": [0.01, 0.1, 0.3],
                "num_leaves": [31, 63, 127],
                "max_depth": [-1, 10, 20, 30],
                "min_child_samples": [5, 10, 20],
                "subsample": [0.6, 0.8, 1.0],
                "colsample_bytree": [0.6, 0.8, 1.0]
            },
            "neural_network": {
                "hidden_layer_sizes": [(50,), (100,), (100, 50), (100, 100)],
                "activation": ["relu", "tanh"],
                "solver": ["adam", "sgd"],
                "alpha": [0.0001, 0.001, 0.01],
                "learning_rate": ["constant", "adaptive"]
            }
        }
        
        return spaces.get(model_name, {})

# Hyperparameter Optimization
class HyperparameterOptimizer:
    """Optimizes hyperparameters using Optuna"""
    
    def __init__(self, model_name: str, model_class: Any, problem_type: ProblemType, optimization_metric: str):
        self.model_name = model_name
        self.model_class = model_class
        self.problem_type = problem_type
        self.optimization_metric = optimization_metric
        self.best_params = None
        self.best_score = None
    
    def objective(self, trial, X_train, y_train, X_val, y_val):
        """Optuna objective function"""
        # Get hyperparameter suggestions based on model type
        params = self._get_trial_params(trial)
        
        # Create and train model
        if self.model_name == "xgboost":
            model = self.model_class(**params, random_state=42, use_label_encoder=False, eval_metric='logloss' if self.problem_type == ProblemType.CLASSIFICATION else 'rmse')
        else:
            model = self.model_class(**params, random_state=42)
        
        model.fit(X_train, y_train)
        
        # Make predictions
        if self.problem_type == ProblemType.CLASSIFICATION:
            y_pred = model.predict(X_val)
            score = self._calculate_classification_metric(y_val, y_pred)
        else:
            y_pred = model.predict(X_val)
            score = self._calculate_regression_metric(y_val, y_pred)
        
        return score
    
    def optimize(self, X_train, y_train, X_val, y_val, n_trials: int = 50):
        """Run hyperparameter optimization"""
        study = optuna.create_study(
            direction="maximize" if self.optimization_metric in ["accuracy", "precision", "recall", "f1_score", "auc_roc", "r2"] else "minimize",
            sampler=TPESampler(seed=42)
        )
        
        study.optimize(
            lambda trial: self.objective(trial, X_train, y_train, X_val, y_val),
            n_trials=n_trials
        )
        
        self.best_params = study.best_params
        self.best_score = study.best_value
        
        return self.best_params, self.best_score
    
    def _get_trial_params(self, trial):
        """Get hyperparameter suggestions for the trial"""
        params = {}
        
        if self.model_name == "logistic_regression":
            params["C"] = trial.suggest_loguniform("C", 0.001, 100)
            params["penalty"] = trial.suggest_categorical("penalty", ["l1", "l2"])
            params["solver"] = "liblinear" if params["penalty"] == "l1" else "lbfgs"
            
        elif self.model_name in ["random_forest", "gradient_boosting"]:
            params["n_estimators"] = trial.suggest_int("n_estimators", 50, 300)
            params["max_depth"] = trial.suggest_int("max_depth", 3, 30)
            params["min_samples_split"] = trial.suggest_int("min_samples_split", 2, 20)
            params["min_samples_leaf"] = trial.suggest_int("min_samples_leaf", 1, 10)
            
            if self.model_name == "random_forest":
                params["max_features"] = trial.suggest_categorical("max_features", ["sqrt", "log2", None])
            else:
                params["learning_rate"] = trial.suggest_loguniform("learning_rate", 0.01, 0.3)
                
        elif self.model_name == "xgboost":
            params["n_estimators"] = trial.suggest_int("n_estimators", 50, 300)
            params["learning_rate"] = trial.suggest_loguniform("learning_rate", 0.01, 0.3)
            params["max_depth"] = trial.suggest_int("max_depth", 3, 10)
            params["min_child_weight"] = trial.suggest_int("min_child_weight", 1, 7)
            params["gamma"] = trial.suggest_uniform("gamma", 0, 0.5)
            params["subsample"] = trial.suggest_uniform("subsample", 0.5, 1.0)
            params["colsample_bytree"] = trial.suggest_uniform("colsample_bytree", 0.5, 1.0)
            
        elif self.model_name == "lightgbm":
            params["n_estimators"] = trial.suggest_int("n_estimators", 50, 300)
            params["learning_rate"] = trial.suggest_loguniform("learning_rate", 0.01, 0.3)
            params["num_leaves"] = trial.suggest_int("num_leaves", 20, 300)
            params["max_depth"] = trial.suggest_int("max_depth", -1, 30)
            params["min_child_samples"] = trial.suggest_int("min_child_samples", 5, 30)
            params["subsample"] = trial.suggest_uniform("subsample", 0.5, 1.0)
            params["colsample_bytree"] = trial.suggest_uniform("colsample_bytree", 0.5, 1.0)
            params["verbosity"] = -1
            
        elif self.model_name == "neural_network":
            n_layers = trial.suggest_int("n_layers", 1, 3)
            layers = []
            for i in range(n_layers):
                layers.append(trial.suggest_int(f"n_units_l{i}", 32, 256))
            params["hidden_layer_sizes"] = tuple(layers)
            params["activation"] = trial.suggest_categorical("activation", ["relu", "tanh"])
            params["solver"] = trial.suggest_categorical("solver", ["adam", "sgd"])
            params["alpha"] = trial.suggest_loguniform("alpha", 0.0001, 0.1)
            params["learning_rate"] = trial.suggest_categorical("learning_rate", ["constant", "adaptive"])
            params["max_iter"] = 1000
        
        return params
    
    def _calculate_classification_metric(self, y_true, y_pred):
        """Calculate classification metric"""
        if self.optimization_metric == "accuracy":
            return accuracy_score(y_true, y_pred)
        elif self.optimization_metric == "precision":
            return precision_score(y_true, y_pred, average='weighted')
        elif self.optimization_metric == "recall":
            return recall_score(y_true, y_pred, average='weighted')
        elif self.optimization_metric == "f1_score":
            return f1_score(y_true, y_pred, average='weighted')
        else:
            return accuracy_score(y_true, y_pred)
    
    def _calculate_regression_metric(self, y_true, y_pred):
        """Calculate regression metric"""
        if self.optimization_metric == "mse":
            return -mean_squared_error(y_true, y_pred)
        elif self.optimization_metric == "rmse":
            return -np.sqrt(mean_squared_error(y_true, y_pred))
        elif self.optimization_metric == "r2":
            return r2_score(y_true, y_pred)
        else:
            return -mean_squared_error(y_true, y_pred)

# AutoML Engine
class AutoMLEngine:
    """Main AutoML engine for automated model training"""
    
    def __init__(self, config: Dict[str, Any]):
        self.config = config
        self.db_session = None
        self.mlflow_tracking_uri = config.get("mlflow_tracking_uri", "sqlite:///mlflow.db")
        mlflow.set_tracking_uri(self.mlflow_tracking_uri)
    
    async def initialize(self):
        """Initialize AutoML engine"""
        # Initialize database
        engine = create_engine(self.config["database_url"])
        Base.metadata.create_all(engine)
        Session = sessionmaker(bind=engine)
        self.db_session = Session()
        
        logger.info("AutoML engine initialized")
    
    async def run_experiment(self, request: AutoMLRequest) -> str:
        """Run an AutoML experiment"""
        experiment_id = str(uuid4())
        
        # Create experiment record
        experiment = AutoMLExperiment(
            id=experiment_id,
            name=request.experiment_name,
            problem_type=request.problem_type.value if request.problem_type else "auto",
            target_column=request.target_column,
            optimization_metric=request.optimization_metric.value if request.optimization_metric else "auto",
            feature_columns=request.feature_columns,
            status="running",
            metadata=request.metadata
        )
        
        self.db_session.add(experiment)
        self.db_session.commit()
        
        # Run experiment in background
        asyncio.create_task(self._run_experiment_async(experiment_id, request))
        
        return experiment_id
    
    async def _run_experiment_async(self, experiment_id: str, request: AutoMLRequest):
        """Run experiment asynchronously"""
        try:
            # Load data
            if request.dataset_path:
                df = pd.read_csv(request.dataset_path)
            elif request.dataset:
                df = pd.DataFrame(request.dataset)
            else:
                raise ValueError("No dataset provided")
            
            # Prepare features and target
            if request.feature_columns:
                X = df[request.feature_columns]
            else:
                X = df.drop(columns=[request.target_column])
            y = df[request.target_column]
            
            # Detect problem type if not specified
            if not request.problem_type:
                problem_type = self._detect_problem_type(y)
            else:
                problem_type = request.problem_type
            
            # Set optimization metric if not specified
            if not request.optimization_metric:
                optimization_metric = self._get_default_metric(problem_type)
            else:
                optimization_metric = request.optimization_metric.value
            
            # Update experiment
            experiment = self.db_session.query(AutoMLExperiment).get(experiment_id)
            experiment.problem_type = problem_type.value
            experiment.optimization_metric = optimization_metric
            experiment.dataset_info = {
                "n_samples": len(df),
                "n_features": X.shape[1],
                "feature_names": list(X.columns)
            }
            self.db_session.commit()
            
            # Preprocess data
            X_processed, preprocessor = self._preprocess_data(X, problem_type)
            
            # Split data
            X_train, X_test, y_train, y_test = train_test_split(
                X_processed, y, test_size=request.test_size, random_state=request.random_state
            )
            
            # Further split train into train and validation
            X_train_split, X_val, y_train_split, y_val = train_test_split(
                X_train, y_train, test_size=0.2, random_state=request.random_state
            )
            
            # Get candidate models
            models = ModelSelector.get_models_for_problem(
                problem_type, len(X_train), X_train.shape[1]
            )
            
            # Train and evaluate models
            best_model = None
            best_score = float('-inf') if optimization_metric in ["accuracy", "precision", "recall", "f1_score", "r2"] else float('inf')
            best_model_id = None
            
            with mlflow.start_run(run_name=f"automl_{experiment_id}"):
                for model_name, model_class in models:
                    logger.info(f"Training {model_name}")
                    
                    # Optimize hyperparameters
                    optimizer = HyperparameterOptimizer(
                        model_name, model_class.__class__, problem_type, optimization_metric
                    )
                    
                    n_trials = min(request.n_trials // len(models), 50)
                    best_params, val_score = optimizer.optimize(
                        X_train_split, y_train_split, X_val, y_val, n_trials
                    )
                    
                    # Train final model with best parameters
                    if model_name == "xgboost":
                        final_model = model_class.__class__(**best_params, random_state=42, use_label_encoder=False, eval_metric='logloss' if problem_type == ProblemType.CLASSIFICATION else 'rmse')
                    else:
                        final_model = model_class.__class__(**best_params, random_state=42)
                    
                    final_model.fit(X_train, y_train)
                    
                    # Evaluate on test set
                    y_pred = final_model.predict(X_test)
                    test_score = self._calculate_metric(y_test, y_pred, problem_type, optimization_metric)
                    
                    # Save model candidate
                    model_id = str(uuid4())
                    model_path = os.path.join(self.config.get("model_storage_path", "/tmp/models"), f"{model_id}.pkl")
                    os.makedirs(os.path.dirname(model_path), exist_ok=True)
                    joblib.dump(final_model, model_path)
                    
                    candidate = ModelCandidate(
                        id=model_id,
                        experiment_id=experiment_id,
                        model_type=model_name,
                        framework=self._get_framework(model_name),
                        hyperparameters=best_params,
                        cross_val_score=val_score,
                        test_score=test_score,
                        model_path=model_path,
                        metrics={
                            "validation_score": val_score,
                            "test_score": test_score
                        }
                    )
                    
                    self.db_session.add(candidate)
                    self.db_session.commit()
                    
                    # Track with MLflow
                    mlflow.log_params(best_params)
                    mlflow.log_metric(f"{model_name}_val_score", val_score)
                    mlflow.log_metric(f"{model_name}_test_score", test_score)
                    
                    # Update best model
                    if self._is_better_score(test_score, best_score, optimization_metric):
                        best_score = test_score
                        best_model = final_model
                        best_model_id = model_id
                
                # Update experiment with best model
                experiment = self.db_session.query(AutoMLExperiment).get(experiment_id)
                experiment.best_model_id = best_model_id
                experiment.best_score = best_score
                experiment.status = "completed"
                experiment.completed_at = datetime.utcnow()
                self.db_session.commit()
                
                # Log best model to MLflow
                mlflow.sklearn.log_model(best_model, "best_model")
                mlflow.log_metric("best_score", best_score)
            
            logger.info(f"Experiment {experiment_id} completed successfully")
            
        except Exception as e:
            logger.error(f"Experiment {experiment_id} failed: {str(e)}")
            experiment = self.db_session.query(AutoMLExperiment).get(experiment_id)
            experiment.status = "failed"
            experiment.metadata = {**experiment.metadata, "error": str(e)}
            self.db_session.commit()
    
    def _detect_problem_type(self, y: pd.Series) -> ProblemType:
        """Detect problem type from target variable"""
        if y.dtype == 'object' or y.nunique() < 10:
            return ProblemType.CLASSIFICATION
        else:
            return ProblemType.REGRESSION
    
    def _get_default_metric(self, problem_type: ProblemType) -> str:
        """Get default optimization metric for problem type"""
        if problem_type == ProblemType.CLASSIFICATION:
            return "f1_score"
        else:
            return "rmse"
    
    def _preprocess_data(self, X: pd.DataFrame, problem_type: ProblemType) -> Tuple[np.ndarray, Any]:
        """Preprocess features"""
        # Handle categorical variables
        categorical_columns = X.select_dtypes(include=['object']).columns
        numeric_columns = X.select_dtypes(include=['number']).columns
        
        # Convert categorical to numeric
        X_processed = X.copy()
        for col in categorical_columns:
            le = LabelEncoder()
            X_processed[col] = le.fit_transform(X_processed[col].astype(str))
        
        # Scale numeric features
        scaler = StandardScaler()
        X_processed[numeric_columns] = scaler.fit_transform(X_processed[numeric_columns])
        
        return X_processed.values, scaler
    
    def _calculate_metric(self, y_true, y_pred, problem_type: ProblemType, metric_name: str) -> float:
        """Calculate evaluation metric"""
        if problem_type == ProblemType.CLASSIFICATION:
            if metric_name == "accuracy":
                return accuracy_score(y_true, y_pred)
            elif metric_name == "precision":
                return precision_score(y_true, y_pred, average='weighted')
            elif metric_name == "recall":
                return recall_score(y_true, y_pred, average='weighted')
            elif metric_name == "f1_score":
                return f1_score(y_true, y_pred, average='weighted')
            else:
                return accuracy_score(y_true, y_pred)
        else:
            if metric_name == "mse":
                return mean_squared_error(y_true, y_pred)
            elif metric_name == "rmse":
                return np.sqrt(mean_squared_error(y_true, y_pred))
            elif metric_name == "r2":
                return r2_score(y_true, y_pred)
            else:
                return mean_squared_error(y_true, y_pred)
    
    def _is_better_score(self, new_score: float, best_score: float, metric_name: str) -> bool:
        """Check if new score is better than best score"""
        if metric_name in ["accuracy", "precision", "recall", "f1_score", "r2"]:
            return new_score > best_score
        else:
            return new_score < best_score
    
    def _get_framework(self, model_name: str) -> str:
        """Get framework for model"""
        if model_name == "xgboost":
            return ModelFramework.XGBOOST.value
        elif model_name == "lightgbm":
            return ModelFramework.LIGHTGBM.value
        elif model_name == "neural_network":
            return ModelFramework.NEURAL_NET.value
        else:
            return ModelFramework.SKLEARN.value
    
    async def get_experiment_status(self, experiment_id: str) -> ExperimentStatusResponse:
        """Get status of an experiment"""
        experiment = self.db_session.query(AutoMLExperiment).get(experiment_id)
        if not experiment:
            raise ValueError(f"Experiment {experiment_id} not found")
        
        # Count models evaluated
        n_models = self.db_session.query(ModelCandidate).filter(
            ModelCandidate.experiment_id == experiment_id
        ).count()
        
        # Calculate elapsed time
        elapsed_time = None
        if experiment.created_at:
            end_time = experiment.completed_at or datetime.utcnow()
            elapsed_time = (end_time - experiment.created_at).total_seconds() / 60
        
        return ExperimentStatusResponse(
            experiment_id=experiment.id,
            name=experiment.name,
            status=experiment.status,
            problem_type=experiment.problem_type,
            optimization_metric=experiment.optimization_metric,
            best_model_id=experiment.best_model_id,
            best_score=experiment.best_score,
            n_models_evaluated=n_models,
            elapsed_time_minutes=elapsed_time,
            created_at=experiment.created_at,
            completed_at=experiment.completed_at
        )

# FastAPI Application
app = FastAPI(title="TracSeq AutoML Service", version="1.0.0")

# Global AutoML engine instance
automl_engine = None

@app.on_event("startup")
async def startup_event():
    """Initialize AutoML engine on startup"""
    global automl_engine
    
    config = {
        "database_url": "postgresql://ml_user:ml_pass@localhost:5436/ml_platform",
        "model_storage_path": "/models/automl",
        "mlflow_tracking_uri": "postgresql://ml_user:ml_pass@localhost:5436/mlflow"
    }
    
    automl_engine = AutoMLEngine(config)
    await automl_engine.initialize()

@app.post("/experiments", response_model=AutoMLResponse)
async def create_experiment(request: AutoMLRequest):
    """Create and run an AutoML experiment"""
    experiment_id = await automl_engine.run_experiment(request)
    return AutoMLResponse(
        experiment_id=experiment_id,
        status="running",
        message=f"Experiment {request.experiment_name} started successfully"
    )

@app.get("/experiments/{experiment_id}", response_model=ExperimentStatusResponse)
async def get_experiment_status(experiment_id: str):
    """Get status of an AutoML experiment"""
    return await automl_engine.get_experiment_status(experiment_id)

@app.get("/health")
async def health_check():
    """Health check endpoint"""
    return {"status": "healthy", "timestamp": datetime.utcnow().isoformat()}

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8096)