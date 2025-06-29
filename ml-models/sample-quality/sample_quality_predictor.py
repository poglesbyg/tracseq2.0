# TracSeq 2.0 - Sample Quality Predictor
# Machine learning model for predicting sample quality in laboratory workflows

import numpy as np
import pandas as pd
from sklearn.ensemble import RandomForestRegressor, GradientBoostingRegressor
from sklearn.preprocessing import StandardScaler, LabelEncoder
from sklearn.model_selection import train_test_split, cross_val_score
from sklearn.metrics import mean_squared_error, r2_score, mean_absolute_error
import mlflow
import mlflow.sklearn
from datetime import datetime, timedelta
import joblib
from typing import Dict, Any, List, Tuple, Optional

class SampleQualityPredictor:
    """
    Predicts sample quality based on:
    - Collection parameters
    - Storage conditions
    - Processing timeline
    - Sample characteristics
    """
    
    def __init__(self):
        self.model = None
        self.scaler = StandardScaler()
        self.label_encoders = {}
        self.feature_names = [
            'collection_method',
            'sample_type',
            'volume_ml',
            'collection_time_hours_ago',
            'storage_temperature',
            'temperature_fluctuation',
            'processing_delay_hours',
            'collection_site',
            'patient_age',
            'hemolysis_level',
            'lipemia_level',
            'container_type',
            'preservative_used',
            'transport_duration_hours',
            'centrifugation_time'
        ]
        self.categorical_features = [
            'collection_method',
            'sample_type',
            'collection_site',
            'container_type',
            'preservative_used'
        ]
    
    def prepare_features(self, df: pd.DataFrame) -> np.ndarray:
        """Prepare features for model training/prediction"""
        # Create a copy to avoid modifying original
        df_processed = df.copy()
        
        # Feature engineering
        df_processed['collection_age_category'] = pd.cut(
            df_processed['collection_time_hours_ago'],
            bins=[0, 6, 24, 72, float('inf')],
            labels=['fresh', 'same_day', 'recent', 'old']
        )
        
        df_processed['temperature_risk'] = (
            abs(df_processed['storage_temperature'] - df_processed['optimal_temperature']) +
            df_processed['temperature_fluctuation']
        )
        
        df_processed['volume_adequate'] = (
            df_processed['volume_ml'] >= df_processed['required_volume_ml']
        ).astype(int)
        
        # Encode categorical variables
        for col in self.categorical_features:
            if col in df_processed.columns:
                if col not in self.label_encoders:
                    self.label_encoders[col] = LabelEncoder()
                    df_processed[col] = self.label_encoders[col].fit_transform(
                        df_processed[col].astype(str)
                    )
                else:
                    # Handle unseen categories
                    df_processed[col] = df_processed[col].apply(
                        lambda x: self.label_encoders[col].transform([x])[0]
                        if x in self.label_encoders[col].classes_ else -1
                    )
        
        # Additional categorical encoding
        if 'collection_age_category' in df_processed.columns:
            if 'collection_age_category' not in self.label_encoders:
                self.label_encoders['collection_age_category'] = LabelEncoder()
                df_processed['collection_age_category'] = self.label_encoders[
                    'collection_age_category'
                ].fit_transform(df_processed['collection_age_category'])
            else:
                df_processed['collection_age_category'] = self.label_encoders[
                    'collection_age_category'
                ].transform(df_processed['collection_age_category'])
        
        # Select and order features
        feature_columns = [
            'collection_method', 'sample_type', 'volume_ml',
            'collection_time_hours_ago', 'storage_temperature',
            'temperature_fluctuation', 'processing_delay_hours',
            'collection_site', 'patient_age', 'hemolysis_level',
            'lipemia_level', 'container_type', 'preservative_used',
            'transport_duration_hours', 'centrifugation_time',
            'collection_age_category', 'temperature_risk', 'volume_adequate'
        ]
        
        # Ensure all features are present
        for col in feature_columns:
            if col not in df_processed.columns:
                df_processed[col] = 0
        
        return df_processed[feature_columns].values
    
    def train(self, X: np.ndarray, y: np.ndarray, experiment_name: str = "sample_quality_prediction"):
        """Train the sample quality prediction model"""
        with mlflow.start_run(run_name=f"sample_quality_{datetime.now().strftime('%Y%m%d_%H%M%S')}"):
            # Log parameters
            mlflow.log_param("model_type", "GradientBoostingRegressor")
            mlflow.log_param("n_features", X.shape[1])
            mlflow.log_param("n_samples", X.shape[0])
            
            # Scale features
            X_scaled = self.scaler.fit_transform(X)
            
            # Split data
            X_train, X_test, y_train, y_test = train_test_split(
                X_scaled, y, test_size=0.2, random_state=42
            )
            
            # Train model
            self.model = GradientBoostingRegressor(
                n_estimators=200,
                learning_rate=0.1,
                max_depth=5,
                min_samples_split=5,
                min_samples_leaf=3,
                subsample=0.8,
                random_state=42
            )
            
            self.model.fit(X_train, y_train)
            
            # Evaluate
            y_pred = self.model.predict(X_test)
            mse = mean_squared_error(y_test, y_pred)
            rmse = np.sqrt(mse)
            mae = mean_absolute_error(y_test, y_pred)
            r2 = r2_score(y_test, y_pred)
            
            # Cross-validation
            cv_scores = cross_val_score(
                self.model, X_scaled, y, cv=5, scoring='r2'
            )
            
            # Log metrics
            mlflow.log_metric("mse", mse)
            mlflow.log_metric("rmse", rmse)
            mlflow.log_metric("mae", mae)
            mlflow.log_metric("r2", r2)
            mlflow.log_metric("cv_r2_mean", cv_scores.mean())
            mlflow.log_metric("cv_r2_std", cv_scores.std())
            
            # Feature importance
            feature_importance = pd.DataFrame({
                'feature': [f"feature_{i}" for i in range(X.shape[1])],
                'importance': self.model.feature_importances_
            }).sort_values('importance', ascending=False)
            
            # Log feature importance
            for idx, row in feature_importance.head(10).iterrows():
                mlflow.log_metric(f"importance_{row['feature']}", row['importance'])
            
            # Log model
            mlflow.sklearn.log_model(self.model, "model")
            
            # Save artifacts
            joblib.dump(self.scaler, "scaler.pkl")
            mlflow.log_artifact("scaler.pkl")
            
            joblib.dump(self.label_encoders, "label_encoders.pkl")
            mlflow.log_artifact("label_encoders.pkl")
            
            print(f"Model trained successfully:")
            print(f"  R² Score: {r2:.4f}")
            print(f"  RMSE: {rmse:.4f}")
            print(f"  MAE: {mae:.4f}")
            
            return {
                "r2": r2,
                "rmse": rmse,
                "mae": mae,
                "cv_r2_mean": cv_scores.mean()
            }
    
    def predict(self, features: Dict[str, Any]) -> Dict[str, Any]:
        """Predict sample quality for a single sample"""
        if self.model is None:
            raise ValueError("Model not trained yet")
        
        # Convert to DataFrame
        df = pd.DataFrame([features])
        
        # Prepare features
        X = self.prepare_features(df)
        X_scaled = self.scaler.transform(X)
        
        # Make prediction
        quality_score = float(self.model.predict(X_scaled)[0])
        
        # Determine quality category and recommendations
        if quality_score >= 0.9:
            category = "excellent"
            recommendations = ["Sample is in excellent condition for all analyses"]
        elif quality_score >= 0.7:
            category = "good"
            recommendations = [
                "Sample is suitable for most analyses",
                "Consider priority processing for sensitive assays"
            ]
        elif quality_score >= 0.5:
            category = "fair"
            recommendations = [
                "Sample quality is compromised",
                "Limit to routine analyses only",
                "Document quality concerns in results"
            ]
        else:
            category = "poor"
            recommendations = [
                "Sample quality is severely compromised",
                "Consider recollection if possible",
                "Results may be unreliable"
            ]
        
        # Add specific recommendations based on features
        if features.get('temperature_fluctuation', 0) > 5:
            recommendations.append("Improve temperature control during storage")
        
        if features.get('processing_delay_hours', 0) > 24:
            recommendations.append("Reduce processing delays to maintain quality")
        
        if features.get('hemolysis_level', 0) > 2:
            recommendations.append("Review collection technique to reduce hemolysis")
        
        return {
            "quality_score": quality_score,
            "quality_category": category,
            "confidence": self._calculate_prediction_confidence(X_scaled),
            "recommendations": recommendations,
            "risk_factors": self._identify_risk_factors(features)
        }
    
    def _calculate_prediction_confidence(self, X_scaled: np.ndarray) -> float:
        """Calculate confidence in prediction"""
        # Use prediction variance from ensemble
        if hasattr(self.model, 'estimators_'):
            predictions = np.array([
                estimator.predict(X_scaled)[0]
                for estimator in self.model.estimators_
            ])
            std_dev = np.std(predictions)
            # Convert to confidence (lower std = higher confidence)
            confidence = max(0.0, min(1.0, 1.0 - (std_dev / 0.5)))
            return float(confidence)
        return 0.85
    
    def _identify_risk_factors(self, features: Dict[str, Any]) -> List[str]:
        """Identify key risk factors affecting sample quality"""
        risk_factors = []
        
        if features.get('collection_time_hours_ago', 0) > 72:
            risk_factors.append("Sample age exceeds 72 hours")
        
        if abs(features.get('storage_temperature', -80) - features.get('optimal_temperature', -80)) > 10:
            risk_factors.append("Storage temperature deviation")
        
        if features.get('temperature_fluctuation', 0) > 5:
            risk_factors.append("High temperature fluctuation")
        
        if features.get('hemolysis_level', 0) > 2:
            risk_factors.append("Significant hemolysis detected")
        
        if features.get('volume_ml', 0) < features.get('required_volume_ml', 5):
            risk_factors.append("Insufficient sample volume")
        
        return risk_factors
    
    def batch_predict(self, samples: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
        """Predict quality for multiple samples"""
        results = []
        for sample in samples:
            try:
                result = self.predict(sample)
                result['sample_id'] = sample.get('sample_id', 'unknown')
                results.append(result)
            except Exception as e:
                results.append({
                    'sample_id': sample.get('sample_id', 'unknown'),
                    'error': str(e),
                    'quality_score': None
                })
        return results
    
    def save_model(self, path: str):
        """Save model and preprocessing artifacts"""
        import os
        os.makedirs(path, exist_ok=True)
        
        joblib.dump(self.model, os.path.join(path, "model.pkl"))
        joblib.dump(self.scaler, os.path.join(path, "scaler.pkl"))
        joblib.dump(self.label_encoders, os.path.join(path, "label_encoders.pkl"))
        
        # Save metadata
        metadata = {
            "model_type": "GradientBoostingRegressor",
            "features": self.feature_names,
            "categorical_features": self.categorical_features,
            "created_at": datetime.now().isoformat()
        }
        
        import json
        with open(os.path.join(path, "metadata.json"), "w") as f:
            json.dump(metadata, f, indent=2)
    
    def load_model(self, path: str):
        """Load model and preprocessing artifacts"""
        import os
        
        self.model = joblib.load(os.path.join(path, "model.pkl"))
        self.scaler = joblib.load(os.path.join(path, "scaler.pkl"))
        self.label_encoders = joblib.load(os.path.join(path, "label_encoders.pkl"))

# Generate synthetic training data for demonstration
def generate_training_data(n_samples: int = 1000) -> Tuple[pd.DataFrame, np.ndarray]:
    """Generate synthetic training data for model development"""
    np.random.seed(42)
    
    data = {
        'collection_method': np.random.choice(['venipuncture', 'fingerstick', 'arterial'], n_samples),
        'sample_type': np.random.choice(['blood', 'serum', 'plasma', 'urine'], n_samples),
        'volume_ml': np.random.normal(5, 1.5, n_samples).clip(0.5, 10),
        'collection_time_hours_ago': np.random.exponential(24, n_samples).clip(0, 168),
        'storage_temperature': np.random.normal(-80, 5, n_samples),
        'optimal_temperature': np.full(n_samples, -80),
        'temperature_fluctuation': np.random.exponential(2, n_samples).clip(0, 20),
        'processing_delay_hours': np.random.exponential(12, n_samples).clip(0, 72),
        'collection_site': np.random.choice(['lab_a', 'lab_b', 'lab_c', 'mobile'], n_samples),
        'patient_age': np.random.normal(45, 20, n_samples).clip(0, 100),
        'hemolysis_level': np.random.choice([0, 1, 2, 3, 4], n_samples, p=[0.6, 0.2, 0.1, 0.07, 0.03]),
        'lipemia_level': np.random.choice([0, 1, 2, 3], n_samples, p=[0.7, 0.2, 0.08, 0.02]),
        'container_type': np.random.choice(['edta', 'sst', 'citrate', 'heparin'], n_samples),
        'preservative_used': np.random.choice(['none', 'standard', 'enhanced'], n_samples),
        'transport_duration_hours': np.random.exponential(4, n_samples).clip(0, 24),
        'centrifugation_time': np.random.normal(10, 2, n_samples).clip(5, 20),
        'required_volume_ml': np.full(n_samples, 3)
    }
    
    df = pd.DataFrame(data)
    
    # Calculate quality score based on multiple factors
    quality_scores = []
    for idx, row in df.iterrows():
        # Base quality
        quality = 1.0
        
        # Degrade based on age
        age_factor = np.exp(-row['collection_time_hours_ago'] / 48)
        quality *= age_factor
        
        # Temperature impact
        temp_deviation = abs(row['storage_temperature'] - row['optimal_temperature'])
        temp_factor = np.exp(-temp_deviation / 10)
        quality *= temp_factor
        
        # Temperature fluctuation impact
        fluct_factor = np.exp(-row['temperature_fluctuation'] / 5)
        quality *= fluct_factor
        
        # Processing delay impact
        delay_factor = np.exp(-row['processing_delay_hours'] / 24)
        quality *= delay_factor
        
        # Hemolysis impact
        hemolysis_factor = 1.0 - (row['hemolysis_level'] * 0.15)
        quality *= hemolysis_factor
        
        # Volume adequacy
        volume_factor = min(1.0, row['volume_ml'] / row['required_volume_ml'])
        quality *= volume_factor
        
        # Add some noise
        quality += np.random.normal(0, 0.05)
        quality = np.clip(quality, 0, 1)
        
        quality_scores.append(quality)
    
    return df, np.array(quality_scores)

if __name__ == "__main__":
    # Example usage
    import mlflow
    mlflow.set_tracking_uri("sqlite:///mlflow.db")
    
    # Generate training data
    print("Generating training data...")
    X_df, y = generate_training_data(2000)
    
    # Initialize and train model
    predictor = SampleQualityPredictor()
    X = predictor.prepare_features(X_df)
    
    print("Training model...")
    metrics = predictor.train(X, y)
    
    # Example prediction
    sample = {
        'collection_method': 'venipuncture',
        'sample_type': 'plasma',
        'volume_ml': 4.5,
        'collection_time_hours_ago': 12,
        'storage_temperature': -78,
        'optimal_temperature': -80,
        'temperature_fluctuation': 3,
        'processing_delay_hours': 6,
        'collection_site': 'lab_a',
        'patient_age': 45,
        'hemolysis_level': 1,
        'lipemia_level': 0,
        'container_type': 'edta',
        'preservative_used': 'standard',
        'transport_duration_hours': 2,
        'centrifugation_time': 10,
        'required_volume_ml': 3
    }
    
    result = predictor.predict(sample)
    print(f"\nSample prediction:")
    print(f"  Quality Score: {result['quality_score']:.3f}")
    print(f"  Category: {result['quality_category']}")
    print(f"  Confidence: {result['confidence']:.3f}")
    print(f"  Risk Factors: {result['risk_factors']}")
    print(f"  Recommendations: {result['recommendations']}")