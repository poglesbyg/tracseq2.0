terraform {
  required_version = ">= 1.6.0"
  
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "~> 2.24"
    }
    helm = {
      source  = "hashicorp/helm"
      version = "~> 2.12"
    }
  }
  
  backend "s3" {
    bucket         = "tracseq-terraform-state"
    key            = "prod/terraform.tfstate"
    region         = "us-east-1"
    dynamodb_table = "tracseq-terraform-locks"
    encrypt        = true
  }
}

provider "aws" {
  region = var.aws_region
  
  default_tags {
    tags = {
      Environment = var.environment
      Project     = "TracSeq-2.0"
      ManagedBy   = "Terraform"
      Owner       = var.owner_email
    }
  }
}

# Data sources
data "aws_availability_zones" "available" {
  state = "available"
}

data "aws_caller_identity" "current" {}

# VPC Module
module "vpc" {
  source = "./modules/vpc"
  
  name                = "${var.project_name}-${var.environment}"
  cidr                = var.vpc_cidr
  azs                 = data.aws_availability_zones.available.names
  private_subnets     = var.private_subnets
  public_subnets      = var.public_subnets
  enable_nat_gateway  = true
  enable_vpn_gateway  = false
  enable_dns_hostnames = true
  enable_dns_support   = true
  
  tags = {
    "kubernetes.io/cluster/${local.cluster_name}" = "shared"
  }
  
  public_subnet_tags = {
    "kubernetes.io/cluster/${local.cluster_name}" = "shared"
    "kubernetes.io/role/elb"                      = "1"
  }
  
  private_subnet_tags = {
    "kubernetes.io/cluster/${local.cluster_name}" = "shared"
    "kubernetes.io/role/internal-elb"             = "1"
  }
}

# EKS Cluster Module
module "eks" {
  source = "./modules/eks"
  
  cluster_name    = local.cluster_name
  cluster_version = var.kubernetes_version
  
  vpc_id          = module.vpc.vpc_id
  subnet_ids      = module.vpc.private_subnets
  
  # Node groups
  node_groups = {
    general = {
      desired_capacity = 3
      max_capacity     = 10
      min_capacity     = 2
      
      instance_types = ["t3.large"]
      
      k8s_labels = {
        Environment = var.environment
        NodeType    = "general"
      }
    }
    
    compute = {
      desired_capacity = 2
      max_capacity     = 5
      min_capacity     = 1
      
      instance_types = ["c5.2xlarge"]
      
      k8s_labels = {
        Environment = var.environment
        NodeType    = "compute"
      }
      
      taints = [
        {
          key    = "compute"
          value  = "true"
          effect = "NO_SCHEDULE"
        }
      ]
    }
  }
  
  # OIDC Provider
  enable_irsa = true
  
  # Add-ons
  cluster_addons = {
    coredns = {
      most_recent = true
    }
    kube-proxy = {
      most_recent = true
    }
    vpc-cni = {
      most_recent = true
    }
    aws-ebs-csi-driver = {
      most_recent = true
    }
  }
  
  tags = local.common_tags
}

# RDS PostgreSQL Cluster
module "rds" {
  source = "./modules/rds"
  
  identifier     = "${var.project_name}-${var.environment}"
  engine         = "aurora-postgresql"
  engine_version = "15.4"
  
  instances = {
    1 = {
      instance_class = "db.r6g.large"
    }
    2 = {
      instance_class = "db.r6g.large"
    }
  }
  
  vpc_id              = module.vpc.vpc_id
  subnets             = module.vpc.private_subnets
  allowed_cidr_blocks = module.vpc.private_subnets_cidr_blocks
  
  database_name   = "tracseq"
  master_username = "tracseq_admin"
  
  backup_retention_period = 30
  preferred_backup_window = "03:00-04:00"
  
  enabled_cloudwatch_logs_exports = ["postgresql"]
  
  tags = local.common_tags
}

# ElastiCache Redis Cluster
module "redis" {
  source = "./modules/elasticache"
  
  name                = "${var.project_name}-${var.environment}"
  engine              = "redis"
  node_type           = "cache.r6g.large"
  num_cache_nodes     = 3
  parameter_group_family = "redis7"
  engine_version      = "7.0"
  port                = 6379
  
  vpc_id              = module.vpc.vpc_id
  subnets             = module.vpc.private_subnets
  allowed_cidr_blocks = module.vpc.private_subnets_cidr_blocks
  
  at_rest_encryption_enabled = true
  transit_encryption_enabled = true
  
  apply_immediately = true
  
  tags = local.common_tags
}

# S3 Buckets
module "s3_storage" {
  source = "./modules/s3"
  
  buckets = {
    laboratory-data = {
      versioning = true
      lifecycle_rules = [
        {
          id      = "archive-old-data"
          enabled = true
          
          transition = [
            {
              days          = 90
              storage_class = "STANDARD_IA"
            },
            {
              days          = 180
              storage_class = "GLACIER"
            }
          ]
        }
      ]
    }
    
    sequencing-results = {
      versioning = true
      lifecycle_rules = [
        {
          id      = "archive-results"
          enabled = true
          
          transition = [
            {
              days          = 30
              storage_class = "STANDARD_IA"
            }
          ]
        }
      ]
    }
    
    ml-models = {
      versioning = true
    }
    
    backups = {
      versioning = true
      lifecycle_rules = [
        {
          id      = "delete-old-backups"
          enabled = true
          
          expiration = {
            days = 90
          }
        }
      ]
    }
  }
  
  tags = local.common_tags
}

# IAM Roles for Service Accounts (IRSA)
module "irsa" {
  source = "./modules/irsa"
  
  eks_cluster_id = module.eks.cluster_id
  
  service_accounts = {
    sample-service = {
      namespace = "tracseq"
      policies = [
        module.s3_storage.bucket_policies["laboratory-data"]
      ]
    }
    
    sequencing-service = {
      namespace = "tracseq"
      policies = [
        module.s3_storage.bucket_policies["sequencing-results"]
      ]
    }
    
    ml-service = {
      namespace = "tracseq"
      policies = [
        module.s3_storage.bucket_policies["ml-models"]
      ]
    }
    
    backup-service = {
      namespace = "tracseq"
      policies = [
        module.s3_storage.bucket_policies["backups"]
      ]
    }
  }
  
  tags = local.common_tags
}

# Application Load Balancer
module "alb" {
  source = "./modules/alb"
  
  name            = "${var.project_name}-${var.environment}"
  vpc_id          = module.vpc.vpc_id
  subnets         = module.vpc.public_subnets
  
  enable_deletion_protection = var.environment == "production"
  enable_http2              = true
  
  access_logs = {
    bucket  = module.s3_storage.bucket_names["logs"]
    enabled = true
  }
  
  tags = local.common_tags
}

# Route53 DNS
module "route53" {
  source = "./modules/route53"
  
  domain_name = var.domain_name
  
  records = [
    {
      name    = ""
      type    = "A"
      alias   = {
        name                   = module.alb.dns_name
        zone_id                = module.alb.zone_id
        evaluate_target_health = true
      }
    },
    {
      name    = "api"
      type    = "A"
      alias   = {
        name                   = module.alb.dns_name
        zone_id                = module.alb.zone_id
        evaluate_target_health = true
      }
    }
  ]
  
  tags = local.common_tags
}

# AWS Certificate Manager
module "acm" {
  source = "./modules/acm"
  
  domain_name               = var.domain_name
  subject_alternative_names = ["*.${var.domain_name}"]
  
  zone_id = module.route53.zone_id
  
  tags = local.common_tags
}

# Monitoring and Alerting
module "monitoring" {
  source = "./modules/monitoring"
  
  cluster_name = module.eks.cluster_id
  
  sns_topic_name = "${var.project_name}-${var.environment}-alerts"
  email_addresses = var.alert_email_addresses
  
  # CloudWatch Log Groups
  log_groups = {
    "/aws/eks/${local.cluster_name}/cluster" = {
      retention_in_days = 30
    }
    "/aws/rds/cluster/${module.rds.cluster_id}" = {
      retention_in_days = 30
    }
    "/aws/elasticache/${module.redis.cluster_id}" = {
      retention_in_days = 7
    }
  }
  
  # Alarms
  alarms = {
    high_cpu = {
      metric_name         = "CPUUtilization"
      namespace           = "AWS/EKS"
      statistic           = "Average"
      period              = 300
      evaluation_periods  = 2
      threshold           = 80
      comparison_operator = "GreaterThanThreshold"
      
      dimensions = {
        ClusterName = module.eks.cluster_id
      }
    }
    
    database_cpu = {
      metric_name         = "CPUUtilization"
      namespace           = "AWS/RDS"
      statistic           = "Average"
      period              = 300
      evaluation_periods  = 2
      threshold           = 75
      comparison_operator = "GreaterThanThreshold"
      
      dimensions = {
        DBClusterIdentifier = module.rds.cluster_id
      }
    }
  }
  
  tags = local.common_tags
}

# Outputs
output "vpc_id" {
  description = "The ID of the VPC"
  value       = module.vpc.vpc_id
}

output "eks_cluster_endpoint" {
  description = "Endpoint for EKS control plane"
  value       = module.eks.cluster_endpoint
}

output "database_endpoint" {
  description = "RDS cluster endpoint"
  value       = module.rds.cluster_endpoint
  sensitive   = true
}

output "redis_endpoint" {
  description = "Redis cluster endpoint"
  value       = module.redis.configuration_endpoint
  sensitive   = true
}

output "load_balancer_dns" {
  description = "DNS name of the load balancer"
  value       = module.alb.dns_name
}

# Local variables
locals {
  cluster_name = "${var.project_name}-${var.environment}"
  
  common_tags = {
    Environment = var.environment
    Project     = var.project_name
    ManagedBy   = "Terraform"
  }
}