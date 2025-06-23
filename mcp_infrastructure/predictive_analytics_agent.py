#!/usr/bin/env python3
"""
Predictive Analytics Agent for TracSeq 2.0

This agent provides ML-powered predictions and forecasting for laboratory operations,
including sample processing time prediction, quality outcome forecasting, and resource optimization.
"""

import asyncio
import logging
from typing import Dict, List, Any, Optional, Union, Tuple
from dataclasses import dataclass, field
from datetime import datetime, timedelta
import numpy as np
import pandas as pd
from sklearn.ensemble import RandomForestRegressor, GradientBoostingClassifier
from sklearn.preprocessing import StandardScaler, LabelEncoder
from sklearn.model_selection import train_test_split
from sklearn.metrics import mean_squared_error, accuracy_score
import joblib
import json

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

@dataclass
class PredictionRequest:
    """Request for a prediction"""
    id: str
    prediction_type: str  # processing_time, quality_outcome, resource_demand, failure_risk
    input_data: Dict[str, Any]
    confidence_threshold: float = 0.7
    context: Dict[str, Any] = field(default_factory=dict)

@dataclass
class PredictionResult:
    """Result of a prediction"""
    request_id: str
    prediction_type: str
    prediction: Any
    confidence: float
    explanation: str
    metadata: Dict[str, Any] = field(default_factory=dict)
    generated_at: datetime = field(default_factory=datetime.now)

@dataclass
class ModelPerformance:
    """Model performance metrics"""
    model_name: str
    accuracy: float
    precision: float
    recall: float
    f1_score: float
    last_trained: datetime
    training_samples: int
    validation_score: float

class PredictiveAnalyticsAgent:
    """
    Advanced ML-powered agent for laboratory predictions and analytics
    """
    
    def __init__(self, config: Dict[str, Any]):
        self.config = config
        self.models = {}
        self.scalers = {}
        self.encoders = {}
        self.model_performance = {}
        
        # Data storage
        self.training_data = {
            'processing_time': [],
            'quality_outcomes': [],
            'resource_usage': [],
            'equipment_failures': []
        }
        
        # Prediction cache
        self.prediction_cache = {}
        self.cache_ttl = timedelta(minutes=30)
        
        self._initialize_models()
    
    def _initialize_models(self):
        """Initialize ML models for different prediction types"""
        
        # Processing Time Prediction Model
        self.models['processing_time'] = RandomForestRegressor(
            n_estimators=100,
            max_depth=10,
            random_state=42
        )
        self.scalers['processing_time'] = StandardScaler()
        
        # Quality Outcome Prediction Model
        self.models['quality_outcome'] = GradientBoostingClassifier(
            n_estimators=100,
            max_depth=6,
            random_state=42
        )
        self.scalers['quality_outcome'] = StandardScaler()
        self.encoders['quality_outcome'] = LabelEncoder()
        
        # Resource Demand Prediction Model
        self.models['resource_demand'] = RandomForestRegressor(
            n_estimators=80,
            max_depth=8,
            random_state=42
        )
        self.scalers['resource_demand'] = StandardScaler()
        
        # Equipment Failure Prediction Model
        self.models['failure_risk'] = GradientBoostingClassifier(
            n_estimators=120,
            max_depth=8,
            random_state=42
        )
        self.scalers['failure_risk'] = StandardScaler()
        
        logger.info("Initialized ML models for predictive analytics")
    
    async def predict_processing_time(self, sample_data: Dict[str, Any]) -> PredictionResult:
        """Predict sample processing time based on characteristics"""
        
        request_id = sample_data.get('request_id', 'processing_time_prediction')
        
        try:
            # Extract features for processing time prediction
            features = self._extract_processing_time_features(sample_data)
            
            # Check if model is trained
            if not hasattr(self.models['processing_time'], 'feature_importances_'):
                # Use heuristic-based prediction if model not trained
                predicted_time = self._heuristic_processing_time(sample_data)
                confidence = 0.6  # Lower confidence for heuristic
                explanation = "Prediction based on heuristic rules (model not yet trained)"
            else:
                # Use trained model
                features_scaled = self.scalers['processing_time'].transform([features])
                predicted_time = self.models['processing_time'].predict(features_scaled)[0]
                confidence = self._calculate_prediction_confidence('processing_time', features)
                explanation = "Prediction based on trained Random Forest model"
            
            return PredictionResult(
                request_id=request_id,
                prediction_type='processing_time',
                prediction=max(30, predicted_time),  # Minimum 30 seconds
                confidence=confidence,
                explanation=explanation,
                metadata={
                    'sample_type': sample_data.get('sample_type'),
                    'complexity_score': self._calculate_complexity_score(sample_data),
                    'features_used': list(range(len(features)))
                }
            )
            
        except Exception as e:
            logger.error(f"Processing time prediction failed: {e}")
            return PredictionResult(
                request_id=request_id,
                prediction_type='processing_time',
                prediction=300,  # Default 5 minutes
                confidence=0.3,
                explanation=f"Prediction failed, using default: {str(e)}"
            )
    
    def _extract_processing_time_features(self, sample_data: Dict[str, Any]) -> List[float]:
        """Extract features for processing time prediction"""
        features = []
        
        # Sample type (encoded)
        sample_type = sample_data.get('sample_type', 'DNA')
        type_encoding = {'DNA': 1, 'RNA': 2, 'Protein': 3, 'Other': 4}
        features.append(type_encoding.get(sample_type, 4))
        
        # Sample volume (ml)
        features.append(float(sample_data.get('volume', 1.0)))
        
        # Sample concentration (ng/μl)
        features.append(float(sample_data.get('concentration', 100.0)))
        
        # Quality score (0-100)
        features.append(float(sample_data.get('quality_score', 85.0)))
        
        # Processing complexity (1-5)
        features.append(float(sample_data.get('complexity', 3)))
        
        # Priority level (1-4: low, medium, high, critical)
        priority_map = {'low': 1, 'medium': 2, 'high': 3, 'critical': 4}
        features.append(priority_map.get(sample_data.get('priority', 'medium'), 2))
        
        # Current lab load (0-100)
        features.append(float(sample_data.get('lab_load', 50.0)))
        
        # Time of day (hour)
        features.append(datetime.now().hour)
        
        # Day of week (1-7)
        features.append(datetime.now().weekday() + 1)
        
        return features
    
    def _heuristic_processing_time(self, sample_data: Dict[str, Any]) -> float:
        """Heuristic-based processing time calculation"""
        base_time = 300  # 5 minutes base
        
        # Adjust for sample type
        sample_type = sample_data.get('sample_type', 'DNA')
        type_multipliers = {'DNA': 1.0, 'RNA': 1.3, 'Protein': 1.5, 'Other': 1.2}
        base_time *= type_multipliers.get(sample_type, 1.2)
        
        # Adjust for complexity
        complexity = sample_data.get('complexity', 3)
        base_time *= (1 + (complexity - 1) * 0.2)
        
        # Adjust for priority
        priority = sample_data.get('priority', 'medium')
        priority_multipliers = {'low': 1.2, 'medium': 1.0, 'high': 0.8, 'critical': 0.6}
        base_time *= priority_multipliers.get(priority, 1.0)
        
        # Adjust for lab load
        lab_load = sample_data.get('lab_load', 50.0)
        base_time *= (1 + lab_load / 200.0)  # Higher load = longer time
        
        return base_time
    
    async def predict_quality_outcome(self, sample_data: Dict[str, Any]) -> PredictionResult:
        """Predict quality outcome for a sample"""
        
        request_id = sample_data.get('request_id', 'quality_prediction')
        
        try:
            # Extract features for quality prediction
            features = self._extract_quality_features(sample_data)
            
            if not hasattr(self.models['quality_outcome'], 'feature_importances_'):
                # Use heuristic-based prediction
                quality_score, quality_class = self._heuristic_quality_prediction(sample_data)
                confidence = 0.65
                explanation = "Prediction based on heuristic quality assessment"
            else:
                # Use trained model
                features_scaled = self.scalers['quality_outcome'].transform([features])
                quality_class_encoded = self.models['quality_outcome'].predict(features_scaled)[0]
                quality_class = self.encoders['quality_outcome'].inverse_transform([quality_class_encoded])[0]
                
                # Calculate quality score
                class_scores = {'Poor': 25, 'Fair': 50, 'Good': 75, 'Excellent': 95}
                quality_score = class_scores.get(quality_class, 75)
                
                confidence = self._calculate_prediction_confidence('quality_outcome', features)
                explanation = "Prediction based on trained Gradient Boosting model"
            
            return PredictionResult(
                request_id=request_id,
                prediction_type='quality_outcome',
                prediction={
                    'quality_class': quality_class,
                    'quality_score': quality_score,
                    'pass_probability': confidence
                },
                confidence=confidence,
                explanation=explanation,
                metadata={
                    'risk_factors': self._identify_quality_risks(sample_data),
                    'recommendations': self._generate_quality_recommendations(sample_data, quality_score)
                }
            )
            
        except Exception as e:
            logger.error(f"Quality outcome prediction failed: {e}")
            return PredictionResult(
                request_id=request_id,
                prediction_type='quality_outcome',
                prediction={'quality_class': 'Good', 'quality_score': 75, 'pass_probability': 0.75},
                confidence=0.4,
                explanation=f"Prediction failed, using default: {str(e)}"
            )
    
    def _extract_quality_features(self, sample_data: Dict[str, Any]) -> List[float]:
        """Extract features for quality prediction"""
        features = []
        
        # Sample integrity score
        features.append(float(sample_data.get('integrity_score', 85.0)))
        
        # Purity level
        features.append(float(sample_data.get('purity', 95.0)))
        
        # pH level
        features.append(float(sample_data.get('ph', 7.0)))
        
        # Storage temperature compliance
        temp_compliance = sample_data.get('temp_compliance', True)
        features.append(1.0 if temp_compliance else 0.0)
        
        # Age of sample (days)
        features.append(float(sample_data.get('age_days', 1)))
        
        # Handling quality score
        features.append(float(sample_data.get('handling_score', 90.0)))
        
        # Container quality
        features.append(float(sample_data.get('container_quality', 95.0)))
        
        # Environmental exposure score
        features.append(float(sample_data.get('environmental_exposure', 5.0)))
        
        return features
    
    def _heuristic_quality_prediction(self, sample_data: Dict[str, Any]) -> Tuple[float, str]:
        """Heuristic-based quality prediction"""
        
        score = 85.0  # Base score
        
        # Adjust for integrity
        integrity = sample_data.get('integrity_score', 85.0)
        score = score * (integrity / 100.0)
        
        # Adjust for purity
        purity = sample_data.get('purity', 95.0)
        score = score * (purity / 100.0)
        
        # Adjust for age
        age_days = sample_data.get('age_days', 1)
        if age_days > 7:
            score *= 0.9
        elif age_days > 30:
            score *= 0.7
        
        # Adjust for temperature compliance
        if not sample_data.get('temp_compliance', True):
            score *= 0.8
        
        # Determine quality class
        if score >= 90:
            quality_class = 'Excellent'
        elif score >= 75:
            quality_class = 'Good'
        elif score >= 60:
            quality_class = 'Fair'
        else:
            quality_class = 'Poor'
        
        return score, quality_class
    
    def _identify_quality_risks(self, sample_data: Dict[str, Any]) -> List[str]:
        """Identify potential quality risk factors"""
        risks = []
        
        if sample_data.get('age_days', 1) > 14:
            risks.append("Sample age exceeds recommended storage time")
        
        if not sample_data.get('temp_compliance', True):
            risks.append("Temperature storage requirements not met")
        
        if sample_data.get('integrity_score', 85) < 70:
            risks.append("Low sample integrity score")
        
        if sample_data.get('purity', 95) < 80:
            risks.append("Purity levels below optimal range")
        
        if sample_data.get('ph', 7.0) < 6.5 or sample_data.get('ph', 7.0) > 8.5:
            risks.append("pH levels outside acceptable range")
        
        return risks
    
    def _generate_quality_recommendations(self, sample_data: Dict[str, Any], quality_score: float) -> List[str]:
        """Generate quality improvement recommendations"""
        recommendations = []
        
        if quality_score < 60:
            recommendations.append("Consider re-sampling if possible")
            recommendations.append("Implement additional quality control measures")
        
        if sample_data.get('age_days', 1) > 7:
            recommendations.append("Process sample with high priority due to age")
        
        if not sample_data.get('temp_compliance', True):
            recommendations.append("Verify and correct storage temperature immediately")
        
        if sample_data.get('purity', 95) < 85:
            recommendations.append("Consider purification steps before processing")
        
        return recommendations
    
    async def predict_resource_demand(self, time_horizon: int = 24) -> PredictionResult:
        """Predict resource demand for the next time_horizon hours"""
        
        request_id = f'resource_demand_{time_horizon}h'
        
        try:
            # Get current lab state
            current_state = await self._get_current_lab_state()
            
            # Predict demand based on historical patterns and current state
            predicted_demand = self._calculate_resource_demand(current_state, time_horizon)
            
            confidence = 0.75  # Resource demand is generally predictable
            
            return PredictionResult(
                request_id=request_id,
                prediction_type='resource_demand',
                prediction=predicted_demand,
                confidence=confidence,
                explanation=f"Resource demand prediction for next {time_horizon} hours based on historical patterns",
                metadata={
                    'time_horizon': time_horizon,
                    'current_utilization': current_state.get('utilization', 0.6),
                    'trend_direction': self._calculate_demand_trend()
                }
            )
            
        except Exception as e:
            logger.error(f"Resource demand prediction failed: {e}")
            return PredictionResult(
                request_id=request_id,
                prediction_type='resource_demand',
                prediction={'equipment': 0.7, 'personnel': 0.6, 'consumables': 0.8},
                confidence=0.4,
                explanation=f"Prediction failed, using default: {str(e)}"
            )
    
    async def _get_current_lab_state(self) -> Dict[str, Any]:
        """Get current laboratory state for predictions"""
        # This would integrate with actual lab systems
        # For now, return mock data
        return {
            'utilization': 0.65,
            'active_samples': 25,
            'equipment_status': 'operational',
            'personnel_available': 8,
            'pending_requests': 12
        }
    
    def _calculate_resource_demand(self, current_state: Dict[str, Any], time_horizon: int) -> Dict[str, float]:
        """Calculate predicted resource demand"""
        
        base_utilization = current_state.get('utilization', 0.6)
        pending_requests = current_state.get('pending_requests', 10)
        
        # Simple demand calculation (would be more sophisticated with real data)
        equipment_demand = min(1.0, base_utilization + (pending_requests * 0.02))
        personnel_demand = min(1.0, base_utilization * 0.8 + (pending_requests * 0.01))
        consumables_demand = min(1.0, base_utilization * 1.2 + (pending_requests * 0.03))
        
        # Adjust for time of day and day of week
        hour = datetime.now().hour
        if 9 <= hour <= 17:  # Business hours
            equipment_demand *= 1.2
            personnel_demand *= 1.3
        
        return {
            'equipment': round(equipment_demand, 2),
            'personnel': round(personnel_demand, 2),
            'consumables': round(consumables_demand, 2),
            'storage': round(equipment_demand * 0.8, 2)
        }
    
    def _calculate_demand_trend(self) -> str:
        """Calculate demand trend direction"""
        # This would analyze historical data
        # For now, return based on time patterns
        hour = datetime.now().hour
        if 8 <= hour <= 12:
            return 'increasing'
        elif 13 <= hour <= 17:
            return 'stable'
        else:
            return 'decreasing'
    
    async def predict_equipment_failure(self, equipment_data: Dict[str, Any]) -> PredictionResult:
        """Predict equipment failure probability"""
        
        request_id = equipment_data.get('request_id', 'failure_prediction')
        
        try:
            # Extract failure prediction features
            features = self._extract_failure_features(equipment_data)
            
            if not hasattr(self.models['failure_risk'], 'feature_importances_'):
                # Use heuristic-based prediction
                failure_probability = self._heuristic_failure_prediction(equipment_data)
                confidence = 0.6
                explanation = "Prediction based on heuristic failure analysis"
            else:
                # Use trained model
                features_scaled = self.scalers['failure_risk'].transform([features])
                failure_probability = self.models['failure_risk'].predict_proba(features_scaled)[0][1]
                confidence = self._calculate_prediction_confidence('failure_risk', features)
                explanation = "Prediction based on trained equipment failure model"
            
            # Determine risk level
            if failure_probability > 0.8:
                risk_level = 'Critical'
            elif failure_probability > 0.6:
                risk_level = 'High'
            elif failure_probability > 0.3:
                risk_level = 'Medium'
            else:
                risk_level = 'Low'
            
            return PredictionResult(
                request_id=request_id,
                prediction_type='failure_risk',
                prediction={
                    'failure_probability': round(failure_probability, 3),
                    'risk_level': risk_level,
                    'predicted_failure_time': self._estimate_failure_time(failure_probability)
                },
                confidence=confidence,
                explanation=explanation,
                metadata={
                    'equipment_id': equipment_data.get('equipment_id'),
                    'maintenance_recommendations': self._generate_maintenance_recommendations(failure_probability)
                }
            )
            
        except Exception as e:
            logger.error(f"Equipment failure prediction failed: {e}")
            return PredictionResult(
                request_id=request_id,
                prediction_type='failure_risk',
                prediction={'failure_probability': 0.1, 'risk_level': 'Low'},
                confidence=0.4,
                explanation=f"Prediction failed, using default: {str(e)}"
            )
    
    def _extract_failure_features(self, equipment_data: Dict[str, Any]) -> List[float]:
        """Extract features for equipment failure prediction"""
        features = []
        
        # Equipment age (years)
        features.append(float(equipment_data.get('age_years', 2.0)))
        
        # Usage hours total
        features.append(float(equipment_data.get('total_hours', 5000)))
        
        # Hours since last maintenance
        features.append(float(equipment_data.get('hours_since_maintenance', 200)))
        
        # Average daily usage (hours)
        features.append(float(equipment_data.get('daily_usage', 8.0)))
        
        # Temperature stress score (0-100)
        features.append(float(equipment_data.get('temp_stress', 30)))
        
        # Vibration level (0-100)
        features.append(float(equipment_data.get('vibration_level', 20)))
        
        # Error count last month
        features.append(float(equipment_data.get('error_count', 2)))
        
        # Performance degradation (0-100)
        features.append(float(equipment_data.get('performance_degradation', 5)))
        
        return features
    
    def _heuristic_failure_prediction(self, equipment_data: Dict[str, Any]) -> float:
        """Heuristic-based failure probability calculation"""
        
        base_probability = 0.05  # 5% base failure rate
        
        # Age factor
        age_years = equipment_data.get('age_years', 2.0)
        if age_years > 5:
            base_probability *= 2.0
        elif age_years > 10:
            base_probability *= 4.0
        
        # Maintenance factor
        hours_since_maintenance = equipment_data.get('hours_since_maintenance', 200)
        if hours_since_maintenance > 1000:
            base_probability *= 3.0
        elif hours_since_maintenance > 2000:
            base_probability *= 5.0
        
        # Error count factor
        error_count = equipment_data.get('error_count', 2)
        base_probability *= (1 + error_count * 0.1)
        
        # Performance degradation factor
        degradation = equipment_data.get('performance_degradation', 5)
        base_probability *= (1 + degradation / 100.0)
        
        return min(1.0, base_probability)
    
    def _estimate_failure_time(self, failure_probability: float) -> str:
        """Estimate time until potential failure"""
        if failure_probability > 0.8:
            return "Within 24 hours"
        elif failure_probability > 0.6:
            return "Within 1 week"
        elif failure_probability > 0.3:
            return "Within 1 month"
        else:
            return "More than 3 months"
    
    def _generate_maintenance_recommendations(self, failure_probability: float) -> List[str]:
        """Generate maintenance recommendations based on failure probability"""
        recommendations = []
        
        if failure_probability > 0.8:
            recommendations.extend([
                "Schedule immediate emergency maintenance",
                "Consider equipment replacement",
                "Implement continuous monitoring"
            ])
        elif failure_probability > 0.6:
            recommendations.extend([
                "Schedule urgent maintenance within 48 hours",
                "Increase monitoring frequency",
                "Prepare backup equipment"
            ])
        elif failure_probability > 0.3:
            recommendations.extend([
                "Schedule routine maintenance",
                "Monitor performance metrics",
                "Check calibration settings"
            ])
        else:
            recommendations.append("Continue regular maintenance schedule")
        
        return recommendations
    
    def _calculate_prediction_confidence(self, model_type: str, features: List[float]) -> float:
        """Calculate confidence score for predictions"""
        # This is a simplified confidence calculation
        # In practice, would use more sophisticated methods like prediction intervals
        
        base_confidence = 0.75
        
        # Adjust confidence based on feature completeness
        feature_completeness = len([f for f in features if f != 0]) / len(features)
        confidence = base_confidence * feature_completeness
        
        # Adjust based on model performance if available
        if model_type in self.model_performance:
            model_accuracy = self.model_performance[model_type].accuracy
            confidence = confidence * model_accuracy
        
        return min(0.95, max(0.3, confidence))
    
    def _calculate_complexity_score(self, sample_data: Dict[str, Any]) -> float:
        """Calculate complexity score for a sample"""
        complexity = 1.0
        
        # Sample type complexity
        sample_type = sample_data.get('sample_type', 'DNA')
        type_complexity = {'DNA': 1.0, 'RNA': 1.3, 'Protein': 1.6, 'Other': 1.2}
        complexity *= type_complexity.get(sample_type, 1.2)
        
        # Processing requirements
        if sample_data.get('requires_special_handling', False):
            complexity *= 1.4
        
        if sample_data.get('requires_multiple_steps', False):
            complexity *= 1.3
        
        # Quality requirements
        quality_threshold = sample_data.get('quality_threshold', 80)
        if quality_threshold > 95:
            complexity *= 1.2
        
        return round(complexity, 2)
    
    async def train_models(self, training_data: Dict[str, Any]) -> Dict[str, ModelPerformance]:
        """Train all prediction models with new data"""
        logger.info("Starting model training...")
        
        performance_results = {}
        
        for model_type in self.models.keys():
            try:
                if model_type in training_data and len(training_data[model_type]) > 10:
                    performance = await self._train_single_model(model_type, training_data[model_type])
                    performance_results[model_type] = performance
                    logger.info(f"Trained {model_type} model with accuracy: {performance.accuracy:.3f}")
                else:
                    logger.warning(f"Insufficient data for {model_type} model training")
            except Exception as e:
                logger.error(f"Failed to train {model_type} model: {e}")
        
        return performance_results
    
    async def _train_single_model(self, model_type: str, data: List[Dict[str, Any]]) -> ModelPerformance:
        """Train a single prediction model"""
        
        # Convert data to features and targets
        if model_type == 'processing_time':
            X, y = self._prepare_processing_time_data(data)
        elif model_type == 'quality_outcome':
            X, y = self._prepare_quality_data(data)
        elif model_type == 'resource_demand':
            X, y = self._prepare_resource_data(data)
        elif model_type == 'failure_risk':
            X, y = self._prepare_failure_data(data)
        else:
            raise ValueError(f"Unknown model type: {model_type}")
        
        # Split data
        X_train, X_test, y_train, y_test = train_test_split(X, y, test_size=0.2, random_state=42)
        
        # Scale features
        self.scalers[model_type].fit(X_train)
        X_train_scaled = self.scalers[model_type].transform(X_train)
        X_test_scaled = self.scalers[model_type].transform(X_test)
        
        # Encode labels if classification
        if model_type in ['quality_outcome', 'failure_risk']:
            self.encoders[model_type].fit(y_train)
            y_train_encoded = self.encoders[model_type].transform(y_train)
            y_test_encoded = self.encoders[model_type].transform(y_test)
        else:
            y_train_encoded = y_train
            y_test_encoded = y_test
        
        # Train model
        self.models[model_type].fit(X_train_scaled, y_train_encoded)
        
        # Evaluate model
        y_pred = self.models[model_type].predict(X_test_scaled)
        
        if model_type in ['quality_outcome', 'failure_risk']:
            accuracy = accuracy_score(y_test_encoded, y_pred)
            precision = recall = f1_score = accuracy  # Simplified for this example
        else:
            mse = mean_squared_error(y_test_encoded, y_pred)
            accuracy = 1.0 / (1.0 + mse)  # Convert MSE to accuracy-like metric
            precision = recall = f1_score = accuracy
        
        # Create performance record
        performance = ModelPerformance(
            model_name=model_type,
            accuracy=accuracy,
            precision=precision,
            recall=recall,
            f1_score=f1_score,
            last_trained=datetime.now(),
            training_samples=len(data),
            validation_score=accuracy
        )
        
        self.model_performance[model_type] = performance
        
        return performance
    
    def _prepare_processing_time_data(self, data: List[Dict[str, Any]]) -> Tuple[np.ndarray, np.ndarray]:
        """Prepare data for processing time model training"""
        X = []
        y = []
        
        for sample in data:
            features = self._extract_processing_time_features(sample)
            X.append(features)
            y.append(sample.get('actual_processing_time', 300))
        
        return np.array(X), np.array(y)
    
    def _prepare_quality_data(self, data: List[Dict[str, Any]]) -> Tuple[np.ndarray, np.ndarray]:
        """Prepare data for quality model training"""
        X = []
        y = []
        
        for sample in data:
            features = self._extract_quality_features(sample)
            X.append(features)
            y.append(sample.get('actual_quality_class', 'Good'))
        
        return np.array(X), np.array(y)
    
    def _prepare_resource_data(self, data: List[Dict[str, Any]]) -> Tuple[np.ndarray, np.ndarray]:
        """Prepare data for resource demand model training"""
        # Simplified implementation
        X = []
        y = []
        
        for record in data:
            features = [
                record.get('hour_of_day', 12),
                record.get('day_of_week', 3),
                record.get('current_load', 0.5),
                record.get('pending_samples', 10)
            ]
            X.append(features)
            y.append(record.get('actual_demand', 0.6))
        
        return np.array(X), np.array(y)
    
    def _prepare_failure_data(self, data: List[Dict[str, Any]]) -> Tuple[np.ndarray, np.ndarray]:
        """Prepare data for failure prediction model training"""
        X = []
        y = []
        
        for record in data:
            features = self._extract_failure_features(record)
            X.append(features)
            y.append(1 if record.get('failure_occurred', False) else 0)
        
        return np.array(X), np.array(y)
    
    async def get_model_status(self) -> Dict[str, Any]:
        """Get status of all prediction models"""
        status = {
            'total_models': len(self.models),
            'trained_models': len([m for m in self.models.values() if hasattr(m, 'feature_importances_') or hasattr(m, 'estimators_')]),
            'model_performance': {k: {
                'accuracy': v.accuracy,
                'last_trained': v.last_trained.isoformat(),
                'training_samples': v.training_samples
            } for k, v in self.model_performance.items()},
            'cache_size': len(self.prediction_cache),
            'last_updated': datetime.now().isoformat()
        }
        
        return status

# Example usage
async def main():
    """Example predictive analytics agent usage"""
    
    config = {
        'cache_ttl_minutes': 30,
        'model_retrain_interval': 3600,  # 1 hour
        'confidence_threshold': 0.7
    }
    
    agent = PredictiveAnalyticsAgent(config)
    
    # Example predictions
    sample_data = {
        'sample_type': 'DNA',
        'volume': 2.0,
        'concentration': 150.0,
        'quality_score': 88.0,
        'complexity': 3,
        'priority': 'high',
        'lab_load': 60.0
    }
    
    # Predict processing time
    processing_result = await agent.predict_processing_time(sample_data)
    print(f"Processing time prediction: {processing_result.prediction} seconds (confidence: {processing_result.confidence:.2f})")
    
    # Predict quality outcome
    quality_result = await agent.predict_quality_outcome(sample_data)
    print(f"Quality prediction: {quality_result.prediction}")
    
    # Predict resource demand
    resource_result = await agent.predict_resource_demand(24)
    print(f"Resource demand (24h): {resource_result.prediction}")

if __name__ == "__main__":
    asyncio.run(main())