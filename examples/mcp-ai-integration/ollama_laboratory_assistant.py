#!/usr/bin/env python3
"""
Ollama Laboratory Assistant
Advanced AI integration for laboratory workflows using Ollama.
"""

import asyncio
import json
import logging
from datetime import datetime
from typing import Dict, Any, List, Optional, AsyncGenerator
import httpx
from dataclasses import dataclass

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

@dataclass
class AnalysisContext:
    """Context for laboratory analysis."""
    sample_type: str
    tests_performed: List[str]
    historical_data: Optional[Dict[str, Any]] = None
    patient_info: Optional[Dict[str, Any]] = None
    urgency: str = "routine"

class OllamaLaboratoryAssistant:
    """Advanced AI assistant for laboratory operations."""
    
    def __init__(self, ollama_url: str = "http://localhost:11434", model: str = "llama3.2:3b"):
        self.ollama_url = ollama_url
        self.model = model
        self.conversation_history = []
        self.specialized_prompts = self._load_specialized_prompts()
        
    def _load_specialized_prompts(self) -> Dict[str, str]:
        """Load specialized prompts for different laboratory tasks."""
        return {
            "result_interpretation": """
You are an experienced clinical laboratory scientist. Analyze the following test results and provide:
1. Clinical significance of each result
2. Potential correlations between results
3. Recommendations for follow-up tests if needed
4. Critical values that require immediate attention

Use medical terminology appropriately but also provide clear explanations.
""",
            "quality_control": """
You are a laboratory quality control specialist. Review the following QC data and:
1. Identify any trends or shifts
2. Determine if results are within acceptable limits
3. Suggest corrective actions if needed
4. Evaluate instrument performance

Focus on statistical significance and patient safety.
""",
            "sample_processing": """
You are a laboratory workflow optimization expert. For the given sample information:
1. Recommend optimal processing sequence
2. Identify potential pre-analytical issues
3. Suggest storage conditions
4. Flag any special handling requirements

Consider efficiency, sample integrity, and regulatory compliance.
""",
            "research_analysis": """
You are a research laboratory scientist. Analyze the experimental data and:
1. Identify significant patterns or anomalies
2. Suggest statistical analyses
3. Recommend additional experiments
4. Evaluate data quality and reproducibility

Maintain scientific rigor in your analysis.
"""
        }
        
    async def analyze_results_with_context(
        self, 
        results: Dict[str, Any], 
        context: AnalysisContext
    ) -> Dict[str, Any]:
        """Analyze laboratory results with full context."""
        
        # Build comprehensive prompt
        prompt = self.specialized_prompts["result_interpretation"] + f"""

Patient Context:
- Sample Type: {context.sample_type}
- Tests Performed: {', '.join(context.tests_performed)}
- Urgency: {context.urgency}

Current Results:
{json.dumps(results, indent=2)}

{f"Historical Data: {json.dumps(context.historical_data, indent=2)}" if context.historical_data else "No historical data available"}

Please provide a comprehensive analysis.
"""
        
        # Get AI analysis
        analysis = await self._query_ollama(prompt)
        
        # Extract structured insights
        insights = await self._extract_structured_insights(analysis, results)
        
        return {
            "timestamp": datetime.now().isoformat(),
            "analysis": analysis,
            "structured_insights": insights,
            "context": {
                "sample_type": context.sample_type,
                "urgency": context.urgency
            }
        }
        
    async def stream_real_time_analysis(
        self, 
        data_stream: AsyncGenerator[Dict[str, Any], None]
    ) -> AsyncGenerator[Dict[str, Any], None]:
        """Process streaming data with real-time AI analysis."""
        
        buffer = []
        async for data_point in data_stream:
            buffer.append(data_point)
            
            # Analyze every 10 data points or on specific triggers
            if len(buffer) >= 10 or data_point.get("trigger_analysis"):
                analysis = await self._analyze_buffer(buffer)
                
                yield {
                    "type": "analysis",
                    "data_points": len(buffer),
                    "insights": analysis,
                    "timestamp": datetime.now().isoformat()
                }
                
                # Keep last 5 points for continuity
                buffer = buffer[-5:]
                
    async def generate_clinical_report(
        self, 
        patient_id: str, 
        test_results: List[Dict[str, Any]],
        include_recommendations: bool = True
    ) -> str:
        """Generate a comprehensive clinical report using AI."""
        
        prompt = f"""
Generate a professional clinical laboratory report for patient {patient_id}.

Test Results:
{json.dumps(test_results, indent=2)}

The report should include:
1. Executive Summary
2. Detailed Results with Reference Ranges
3. Clinical Interpretation
{f"4. Recommendations for Follow-up" if include_recommendations else ""}
5. Methodology Notes

Format the report in a clear, professional manner suitable for healthcare providers.
"""
        
        report = await self._query_ollama(prompt, temperature=0.3)  # Lower temperature for consistency
        
        # Add metadata
        return f"""
CLINICAL LABORATORY REPORT
Generated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}
Patient ID: {patient_id}
Report ID: RPT-{datetime.now().strftime('%Y%m%d%H%M%S')}

{report}

[This report was generated with AI assistance and should be reviewed by qualified personnel]
"""
        
    async def optimize_workflow(
        self, 
        pending_samples: List[Dict[str, Any]],
        available_resources: Dict[str, Any]
    ) -> Dict[str, Any]:
        """Use AI to optimize laboratory workflow."""
        
        prompt = self.specialized_prompts["sample_processing"] + f"""

Pending Samples:
{json.dumps(pending_samples, indent=2)}

Available Resources:
- Instruments: {available_resources.get('instruments', [])}
- Staff: {available_resources.get('staff_count', 0)}
- Time Window: {available_resources.get('time_window', '8 hours')}

Optimize the workflow considering:
1. TAT (Turnaround Time) requirements
2. Instrument capacity
3. Sample stability
4. Batch efficiency
5. Priority levels

Provide a detailed schedule and rationale.
"""
        
        optimization = await self._query_ollama(prompt)
        
        # Parse into structured format
        schedule = await self._parse_workflow_schedule(optimization, pending_samples)
        
        return {
            "optimization_plan": optimization,
            "structured_schedule": schedule,
            "estimated_completion": self._estimate_completion_time(schedule),
            "efficiency_score": self._calculate_efficiency_score(schedule, available_resources)
        }
        
    async def detect_anomalies(
        self, 
        current_results: Dict[str, Any],
        historical_data: List[Dict[str, Any]],
        sensitivity: str = "normal"
    ) -> Dict[str, Any]:
        """Detect anomalies in laboratory results using AI."""
        
        prompt = f"""
Analyze the current laboratory results for anomalies by comparing with historical data.

Current Results:
{json.dumps(current_results, indent=2)}

Historical Data (last {len(historical_data)} results):
{json.dumps(historical_data, indent=2)}

Sensitivity Level: {sensitivity}

Identify:
1. Statistical outliers
2. Clinically significant changes
3. Unusual patterns or combinations
4. Potential data quality issues

Provide confidence scores for each finding.
"""
        
        analysis = await self._query_ollama(prompt)
        
        anomalies = {
            "detected_anomalies": [],
            "risk_level": "low",
            "recommended_actions": [],
            "confidence_scores": {}
        }
        
        # Parse AI response for structured anomalies
        # This would include more sophisticated parsing in production
        if "critical" in analysis.lower() or "urgent" in analysis.lower():
            anomalies["risk_level"] = "high"
            
        return {
            "analysis": analysis,
            "anomalies": anomalies,
            "timestamp": datetime.now().isoformat()
        }
        
    async def interactive_consultation(
        self, 
        question: str,
        context: Optional[Dict[str, Any]] = None
    ) -> str:
        """Interactive consultation with the AI assistant."""
        
        # Add context to conversation history
        if context:
            self.conversation_history.append({
                "role": "system",
                "content": f"Context update: {json.dumps(context)}"
            })
            
        # Add user question
        self.conversation_history.append({
            "role": "user",
            "content": question
        })
        
        # Build conversation prompt
        conversation = "\n".join([
            f"{msg['role']}: {msg['content']}" 
            for msg in self.conversation_history[-10:]  # Keep last 10 messages
        ])
        
        prompt = f"""
You are an expert laboratory medicine consultant. 

Previous conversation:
{conversation}

Provide a helpful, accurate response based on laboratory best practices and current medical knowledge.
"""
        
        response = await self._query_ollama(prompt)
        
        # Add to history
        self.conversation_history.append({
            "role": "assistant",
            "content": response
        })
        
        return response
        
    async def _query_ollama(
        self, 
        prompt: str, 
        temperature: float = 0.7,
        stream: bool = False
    ) -> str:
        """Query Ollama API."""
        try:
            async with httpx.AsyncClient() as client:
                response = await client.post(
                    f"{self.ollama_url}/api/generate",
                    json={
                        "model": self.model,
                        "prompt": prompt,
                        "temperature": temperature,
                        "stream": stream
                    },
                    timeout=60.0
                )
                
                if response.status_code == 200:
                    return response.json().get("response", "")
                else:
                    logger.error(f"Ollama API error: {response.status_code}")
                    return ""
                    
        except Exception as e:
            logger.error(f"Error querying Ollama: {e}")
            return ""
            
    async def _extract_structured_insights(
        self, 
        analysis: str, 
        results: Dict[str, Any]
    ) -> Dict[str, Any]:
        """Extract structured insights from AI analysis."""
        # This would use more sophisticated NLP in production
        insights = {
            "critical_findings": [],
            "normal_findings": [],
            "recommendations": [],
            "follow_up_tests": []
        }
        
        # Simple keyword extraction
        if "critical" in analysis.lower() or "urgent" in analysis.lower():
            insights["critical_findings"].append({
                "finding": "Critical values detected",
                "action": "Immediate physician notification required"
            })
            
        return insights
        
    async def _analyze_buffer(self, buffer: List[Dict[str, Any]]) -> Dict[str, Any]:
        """Analyze a buffer of streaming data."""
        summary = {
            "data_points": len(buffer),
            "trends": [],
            "anomalies": []
        }
        
        # Perform analysis
        prompt = f"""
Analyze this sequence of real-time laboratory data:
{json.dumps(buffer, indent=2)}

Identify trends, patterns, and any concerning changes.
"""
        
        analysis = await self._query_ollama(prompt)
        summary["ai_analysis"] = analysis
        
        return summary
        
    async def _parse_workflow_schedule(
        self, 
        optimization: str, 
        samples: List[Dict[str, Any]]
    ) -> List[Dict[str, Any]]:
        """Parse AI optimization into structured schedule."""
        # This would include sophisticated parsing logic
        schedule = []
        
        for idx, sample in enumerate(samples):
            schedule.append({
                "sample_id": sample.get("id"),
                "priority": sample.get("priority", "routine"),
                "scheduled_time": f"T+{idx*15} minutes",
                "instrument": "Auto-analyzer 1",
                "estimated_duration": "15 minutes"
            })
            
        return schedule
        
    def _estimate_completion_time(self, schedule: List[Dict[str, Any]]) -> str:
        """Estimate workflow completion time."""
        # Simple estimation
        total_minutes = len(schedule) * 15
        return f"{total_minutes // 60} hours {total_minutes % 60} minutes"
        
    def _calculate_efficiency_score(
        self, 
        schedule: List[Dict[str, Any]], 
        resources: Dict[str, Any]
    ) -> float:
        """Calculate workflow efficiency score."""
        # Simple scoring algorithm
        return min(0.95, len(schedule) / (resources.get("staff_count", 1) * 10))

# Example usage
async def demonstrate_ai_capabilities():
    """Demonstrate various AI capabilities."""
    assistant = OllamaLaboratoryAssistant()
    
    # Example 1: Analyze results with context
    results = {
        "glucose": {"value": 250, "unit": "mg/dL", "reference": "70-100"},
        "hba1c": {"value": 9.2, "unit": "%", "reference": "<5.7"},
        "creatinine": {"value": 1.8, "unit": "mg/dL", "reference": "0.6-1.2"}
    }
    
    context = AnalysisContext(
        sample_type="blood",
        tests_performed=["glucose", "hba1c", "creatinine"],
        urgency="stat"
    )
    
    analysis = await assistant.analyze_results_with_context(results, context)
    logger.info(f"AI Analysis: {analysis}")
    
    # Example 2: Optimize workflow
    pending_samples = [
        {"id": "S001", "type": "blood", "tests": ["CBC"], "priority": "stat"},
        {"id": "S002", "type": "urine", "tests": ["UA"], "priority": "routine"},
        {"id": "S003", "type": "blood", "tests": ["CMP"], "priority": "urgent"}
    ]
    
    resources = {
        "instruments": ["Auto-analyzer 1", "Auto-analyzer 2"],
        "staff_count": 3,
        "time_window": "4 hours"
    }
    
    optimization = await assistant.optimize_workflow(pending_samples, resources)
    logger.info(f"Workflow optimization: {optimization}")
    
    # Example 3: Interactive consultation
    response = await assistant.interactive_consultation(
        "What are the implications of elevated HbA1c with high creatinine?",
        context={"patient_age": 65, "diabetic": True}
    )
    logger.info(f"Consultation response: {response}")

if __name__ == "__main__":
    asyncio.run(demonstrate_ai_capabilities()) 