# MIR Runtime Deployment Guide

This guide covers deploying the MIR Runtime in different environments, from development to production distributed systems.

## Table of Contents

1. [Development Deployment](#development-deployment)
2. [Production Deployment](#production-deployment)
3. [Distributed Deployment](#distributed-deployment)
4. [Container Deployment](#container-deployment)
5. [Cloud Deployment](#cloud-deployment)
6. [Monitoring and Observability](#monitoring-and-observability)
7. [Security Considerations](#security-considerations)
8. [Troubleshooting](#troubleshooting)

## Development Deployment

### Local Development Setup

For local development, use the development environment configuration:

```rust
use mir_runtime_assembly::{MirRuntime, RuntimeConfig, Environment};

let config = RuntimeConfig::builder()
    .with_environment(Environment::Development {
        auto_reload: true,
        file_watcher_enabled: true,
        aggressive_optimization: false,
    })
    .with_backend("wasm")
    .with_backend("js")
    .with_max_concurrent_executions(4)
    .with_execution_timeout(Duration::from_secs(30))
    .with_sandbox(false) // Disable for easier debugging
    .build()?;

let mut runtime = MirRuntime::new(config).await?;
runtime.start().await?;
```

### Development Tools Integration

#### File Watcher Setup

```rust
use mir_runtime_assembly::DevToolsConfig;

let dev_tools_config = DevToolsConfig {
    file_watcher_enabled: true,
    watch_patterns: vec![
        "src/**/*.rs".to_string(),
        "modules/**/*.mir".to_string(),
    ],
    ignore_patterns: vec![
        "target/**".to_string(),
        ".git/**".to_string(),
    ],
    debounce_ms: 100,
};
```

#### Build Tool Integration

##### Webpack Integration

```javascript
// webpack.config.js
const MirRuntimePlugin = require('@mir/webpack-plugin');

module.exports = {
  plugins: [
    new MirRuntimePlugin({
      runtimeEndpoint: 'http://localhost:8080/mir',
      hotReload: true,
      sourceMap: true,
    }),
  ],
};
```

##### Vite Integration

```javascript
// vite.config.js
import { defineConfig } from 'vite';
import { mirRuntime } from '@mir/vite-plugin';

export default defineConfig({
  plugins: [
    mirRuntime({
      runtimeEndpoint: 'http://localhost:8080/mir',
      hotReload: true,
    }),
  ],
});
```

## Production Deployment

### Production Configuration

```rust
use mir_runtime_assembly::{
    MirRuntime, RuntimeConfig, Environment, UpdatePolicy, ValidationLevel,
    RollbackStrategy, StatePreservationConfig
};

let config = RuntimeConfig::builder()
    .with_environment(Environment::Production {
        require_explicit_deployment: true,
        staged_rollout: true,
        canary_percentage: 10.0,
    })
    .with_hmr_config(HMRConfig {
        update_policy: UpdatePolicy::Manual,
        validation_level: ValidationLevel::Extensive,
        rollback_strategy: RollbackStrategy::Automatic,
        state_preservation: StatePreservationConfig {
            enable_snapshots: true,
            snapshot_interval: Duration::from_secs(300), // 5 minutes
            max_snapshots: 10,
            compress_snapshots: true,
        },
        ..Default::default()
    })
    .with_backend("wasm") // Prefer WASM for production
    .with_max_concurrent_executions(16)
    .with_execution_timeout(Duration::from_secs(60))
    .with_sandbox(true) // Enable sandboxing for security
    .build()?;
```

### Single Node Production

For single-node production deployments:

```rust
let config = RuntimeConfig::builder()
    .with_environment(Environment::Production {
        require_explicit_deployment: true,
        staged_rollout: false, // No staged rollout for single node
        canary_percentage: 0.0,
    })
    .with_runtime_settings(RuntimeSettings {
        max_concurrent_executions: num_cpus::get() * 2,
        execution_timeout: Duration::from_secs(120),
        memory_limits: MemoryLimits {
            max_heap_size: 2 * 1024 * 1024 * 1024, // 2GB
            max_stack_size: 16 * 1024 * 1024,      // 16MB
            pressure_threshold: 0.85,
        },
        gc_settings: GCSettings {
            strategy: GCStrategy::Concurrent,
            trigger_threshold: 0.8,
            max_pause_time: Duration::from_millis(5),
            concurrent: true,
        },
        security: SecuritySettings {
            enable_sandbox: true,
            resource_limits: ResourceLimits {
                max_file_descriptors: 4096,
                max_network_connections: 1000,
                max_cpu_time: Duration::from_secs(300),
                max_memory_per_execution: 512 * 1024 * 1024, // 512MB
            },
            ..Default::default()
        },
        ..Default::default()
    })
    .build()?;
```

## Distributed Deployment

### Multi-Node Cluster Setup

For distributed deployments with multiple nodes:

```rust
use mir_runtime_assembly::{ConsensusConfig, DistributedHMRConfig};

let consensus_config = ConsensusConfig {
    node_id: std::env::var("NODE_ID").unwrap_or_else(|_| "node-1".to_string()),
    cluster_nodes: vec![
        "node-1".to_string(),
        "node-2".to_string(),
        "node-3".to_string(),
    ],
    consensus_timeout: Duration::from_secs(30),
    heartbeat_interval: Duration::from_secs(5),
    election_timeout: Duration::from_secs(15),
    max_retries: 3,
};

let config = RuntimeConfig::builder()
    .with_environment(Environment::Production {
        require_explicit_deployment: true,
        staged_rollout: true,
        canary_percentage: 20.0, // 20% canary deployment
    })
    .with_consensus_config(consensus_config)
    .with_hmr_config(HMRConfig {
        update_policy: UpdatePolicy::Consensus,
        validation_level: ValidationLevel::Extensive,
        consensus_timeout: Duration::from_secs(60),
        ..Default::default()
    })
    .build()?;
```

### Node Discovery and Registration

```rust
// Automatic node discovery using environment variables
fn get_cluster_nodes() -> Vec<String> {
    if let Ok(nodes_env) = std::env::var("CLUSTER_NODES") {
        nodes_env.split(',').map(|s| s.trim().to_string()).collect()
    } else {
        // Fallback to DNS-based discovery
        discover_nodes_via_dns("mir-cluster.internal").unwrap_or_else(|_| {
            vec!["localhost:8080".to_string()]
        })
    }
}

fn discover_nodes_via_dns(service_name: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    // Implementation would use DNS SRV records or similar
    // This is a placeholder
    Ok(vec![
        "node-1.mir-cluster.internal:8080".to_string(),
        "node-2.mir-cluster.internal:8080".to_string(),
        "node-3.mir-cluster.internal:8080".to_string(),
    ])
}
```

### Load Balancing

Use a load balancer to distribute traffic across nodes:

```yaml
# nginx.conf
upstream mir_cluster {
    server node-1.mir-cluster.internal:8080;
    server node-2.mir-cluster.internal:8080;
    server node-3.mir-cluster.internal:8080;
}

server {
    listen 80;
    server_name mir.example.com;
    
    location / {
        proxy_pass http://mir_cluster;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        
        # WebSocket support for hot-reloading
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }
}
```

## Container Deployment

### Docker Setup

#### Dockerfile

```dockerfile
FROM rust:1.70 as builder

WORKDIR /app
COPY . .
RUN cargo build --release --bin mir-runtime

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/mir-runtime /usr/local/bin/mir-runtime

EXPOSE 8080
CMD ["mir-runtime"]
```

#### Docker Compose

```yaml
version: '3.8'

services:
  mir-node-1:
    build: .
    environment:
      - NODE_ID=node-1
      - CLUSTER_NODES=mir-node-1:8080,mir-node-2:8080,mir-node-3:8080
      - RUST_LOG=info
    ports:
      - "8081:8080"
    volumes:
      - ./config:/app/config
      - mir-data-1:/app/data

  mir-node-2:
    build: .
    environment:
      - NODE_ID=node-2
      - CLUSTER_NODES=mir-node-1:8080,mir-node-2:8080,mir-node-3:8080
      - RUST_LOG=info
    ports:
      - "8082:8080"
    volumes:
      - ./config:/app/config
      - mir-data-2:/app/data

  mir-node-3:
    build: .
    environment:
      - NODE_ID=node-3
      - CLUSTER_NODES=mir-node-1:8080,mir-node-2:8080,mir-node-3:8080
      - RUST_LOG=info
    ports:
      - "8083:8080"
    volumes:
      - ./config:/app/config
      - mir-data-3:/app/data

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
    depends_on:
      - mir-node-1
      - mir-node-2
      - mir-node-3

volumes:
  mir-data-1:
  mir-data-2:
  mir-data-3:
```

### Kubernetes Deployment

#### Deployment Manifest

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: mir-runtime
  labels:
    app: mir-runtime
spec:
  replicas: 3
  selector:
    matchLabels:
      app: mir-runtime
  template:
    metadata:
      labels:
        app: mir-runtime
    spec:
      containers:
      - name: mir-runtime
        image: mir-runtime:latest
        ports:
        - containerPort: 8080
        env:
        - name: NODE_ID
          valueFrom:
            fieldRef:
              fieldPath: metadata.name
        - name: CLUSTER_NODES
          value: "mir-runtime-0.mir-runtime-headless:8080,mir-runtime-1.mir-runtime-headless:8080,mir-runtime-2.mir-runtime-headless:8080"
        - name: RUST_LOG
          value: "info"
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "1Gi"
            cpu: "1000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
        volumeMounts:
        - name: config
          mountPath: /app/config
        - name: data
          mountPath: /app/data
      volumes:
      - name: config
        configMap:
          name: mir-runtime-config
      - name: data
        persistentVolumeClaim:
          claimName: mir-runtime-data
```

#### Service Manifest

```yaml
apiVersion: v1
kind: Service
metadata:
  name: mir-runtime
spec:
  selector:
    app: mir-runtime
  ports:
  - port: 80
    targetPort: 8080
  type: LoadBalancer

---
apiVersion: v1
kind: Service
metadata:
  name: mir-runtime-headless
spec:
  clusterIP: None
  selector:
    app: mir-runtime
  ports:
  - port: 8080
    targetPort: 8080
```

## Cloud Deployment

### AWS Deployment

#### ECS Task Definition

```json
{
  "family": "mir-runtime",
  "networkMode": "awsvpc",
  "requiresCompatibilities": ["FARGATE"],
  "cpu": "1024",
  "memory": "2048",
  "executionRoleArn": "arn:aws:iam::account:role/ecsTaskExecutionRole",
  "taskRoleArn": "arn:aws:iam::account:role/mirRuntimeTaskRole",
  "containerDefinitions": [
    {
      "name": "mir-runtime",
      "image": "your-account.dkr.ecr.region.amazonaws.com/mir-runtime:latest",
      "portMappings": [
        {
          "containerPort": 8080,
          "protocol": "tcp"
        }
      ],
      "environment": [
        {
          "name": "RUST_LOG",
          "value": "info"
        }
      ],
      "secrets": [
        {
          "name": "CLUSTER_NODES",
          "valueFrom": "arn:aws:ssm:region:account:parameter/mir/cluster-nodes"
        }
      ],
      "logConfiguration": {
        "logDriver": "awslogs",
        "options": {
          "awslogs-group": "/ecs/mir-runtime",
          "awslogs-region": "us-west-2",
          "awslogs-stream-prefix": "ecs"
        }
      }
    }
  ]
}
```

#### CloudFormation Template

```yaml
AWSTemplateFormatVersion: '2010-09-09'
Description: 'MIR Runtime Cluster'

Parameters:
  VpcId:
    Type: AWS::EC2::VPC::Id
  SubnetIds:
    Type: List<AWS::EC2::Subnet::Id>
  ImageUri:
    Type: String

Resources:
  MirRuntimeCluster:
    Type: AWS::ECS::Cluster
    Properties:
      ClusterName: mir-runtime-cluster

  MirRuntimeService:
    Type: AWS::ECS::Service
    Properties:
      Cluster: !Ref MirRuntimeCluster
      TaskDefinition: !Ref MirRuntimeTaskDefinition
      DesiredCount: 3
      LaunchType: FARGATE
      NetworkConfiguration:
        AwsvpcConfiguration:
          SecurityGroups:
            - !Ref MirRuntimeSecurityGroup
          Subnets: !Ref SubnetIds
          AssignPublicIp: ENABLED
      LoadBalancers:
        - ContainerName: mir-runtime
          ContainerPort: 8080
          TargetGroupArn: !Ref MirRuntimeTargetGroup

  MirRuntimeLoadBalancer:
    Type: AWS::ElasticLoadBalancingV2::LoadBalancer
    Properties:
      Type: application
      Scheme: internet-facing
      SecurityGroups:
        - !Ref MirRuntimeSecurityGroup
      Subnets: !Ref SubnetIds

  MirRuntimeTargetGroup:
    Type: AWS::ElasticLoadBalancingV2::TargetGroup
    Properties:
      Port: 8080
      Protocol: HTTP
      VpcId: !Ref VpcId
      TargetType: ip
      HealthCheckPath: /health
```

### Google Cloud Platform

#### Cloud Run Deployment

```yaml
apiVersion: serving.knative.dev/v1
kind: Service
metadata:
  name: mir-runtime
  annotations:
    run.googleapis.com/ingress: all
spec:
  template:
    metadata:
      annotations:
        autoscaling.knative.dev/minScale: "1"
        autoscaling.knative.dev/maxScale: "10"
        run.googleapis.com/cpu-throttling: "false"
    spec:
      containerConcurrency: 100
      containers:
      - image: gcr.io/project-id/mir-runtime:latest
        ports:
        - containerPort: 8080
        env:
        - name: RUST_LOG
          value: "info"
        - name: CLUSTER_NODES
          valueFrom:
            secretKeyRef:
              name: mir-config
              key: cluster-nodes
        resources:
          limits:
            cpu: "2"
            memory: "2Gi"
          requests:
            cpu: "1"
            memory: "1Gi"
```

### Azure Container Instances

```yaml
apiVersion: 2019-12-01
location: eastus
name: mir-runtime-group
properties:
  containers:
  - name: mir-runtime
    properties:
      image: your-registry.azurecr.io/mir-runtime:latest
      resources:
        requests:
          cpu: 1
          memoryInGb: 2
      ports:
      - port: 8080
        protocol: TCP
      environmentVariables:
      - name: RUST_LOG
        value: info
      - name: NODE_ID
        value: azure-node-1
  osType: Linux
  restartPolicy: Always
  ipAddress:
    type: Public
    ports:
    - protocol: TCP
      port: 8080
tags:
  app: mir-runtime
```

## Monitoring and Observability

### Prometheus Metrics

```yaml
# prometheus.yml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'mir-runtime'
    static_configs:
      - targets: ['mir-node-1:8080', 'mir-node-2:8080', 'mir-node-3:8080']
    metrics_path: /metrics
    scrape_interval: 10s
```

### Grafana Dashboard

```json
{
  "dashboard": {
    "title": "MIR Runtime Dashboard",
    "panels": [
      {
        "title": "Execution Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(mir_executions_total[5m])",
            "legendFormat": "{{backend}} - {{status}}"
          }
        ]
      },
      {
        "title": "Hot Reload Success Rate",
        "type": "stat",
        "targets": [
          {
            "expr": "rate(mir_hot_reloads_total{status=\"success\"}[5m]) / rate(mir_hot_reloads_total[5m]) * 100"
          }
        ]
      },
      {
        "title": "Consensus Latency",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, rate(mir_consensus_latency_seconds_bucket[5m]))",
            "legendFormat": "95th percentile"
          }
        ]
      }
    ]
  }
}
```

### Jaeger Tracing

```yaml
# jaeger-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: jaeger
spec:
  replicas: 1
  selector:
    matchLabels:
      app: jaeger
  template:
    metadata:
      labels:
        app: jaeger
    spec:
      containers:
      - name: jaeger
        image: jaegertracing/all-in-one:latest
        ports:
        - containerPort: 16686
        - containerPort: 14268
        env:
        - name: COLLECTOR_OTLP_ENABLED
          value: "true"
```

## Security Considerations

### Network Security

```rust
// Enable TLS for inter-node communication
let security_config = SecuritySettings {
    enable_tls: true,
    tls_cert_path: "/etc/ssl/certs/mir-runtime.crt".to_string(),
    tls_key_path: "/etc/ssl/private/mir-runtime.key".to_string(),
    tls_ca_path: "/etc/ssl/certs/ca.crt".to_string(),
    ..Default::default()
};
```

### Authentication and Authorization

```rust
// Configure authentication
let auth_config = AuthConfig {
    enable_auth: true,
    auth_provider: AuthProvider::JWT {
        secret_key: std::env::var("JWT_SECRET").expect("JWT_SECRET required"),
        issuer: "mir-runtime".to_string(),
        audience: "mir-cluster".to_string(),
    },
    rbac_enabled: true,
    rbac_config_path: "/etc/mir/rbac.yaml".to_string(),
};
```

### Sandboxing Configuration

```rust
let security_config = SecuritySettings {
    enable_sandbox: true,
    sandbox_type: SandboxType::Strict,
    allowed_syscalls: vec![
        "read", "write", "open", "close", "mmap", "munmap"
    ].into_iter().map(String::from).collect(),
    resource_limits: ResourceLimits {
        max_file_descriptors: 256,
        max_network_connections: 10,
        max_cpu_time: Duration::from_secs(30),
        max_memory_per_execution: 128 * 1024 * 1024, // 128MB
    },
    capability_based: true,
};
```

## Troubleshooting

### Common Issues

#### Runtime Won't Start

1. **Check Configuration**
   ```bash
   mir-runtime --validate-config /path/to/config.json
   ```

2. **Check Dependencies**
   ```bash
   ldd /usr/local/bin/mir-runtime
   ```

3. **Check Permissions**
   ```bash
   ls -la /usr/local/bin/mir-runtime
   chmod +x /usr/local/bin/mir-runtime
   ```

#### Hot-Reloading Fails

1. **Check Module Compatibility**
   ```rust
   let analysis = runtime.analyze_change(old_hash, new_module).await?;
   println!("Compatibility: {:?}", analysis.compatibility);
   ```

2. **Review Migration Functions**
   ```rust
   if analysis.migration_required {
       println!("Migration functions needed: {:?}", analysis.required_migrations);
   }
   ```

#### Distributed Coordination Issues

1. **Check Network Connectivity**
   ```bash
   telnet node-2.mir-cluster.internal 8080
   ```

2. **Check Consensus Configuration**
   ```rust
   let consensus_status = runtime.get_consensus_status().await;
   println!("Consensus health: {:?}", consensus_status);
   ```

3. **Monitor Node Health**
   ```bash
   curl http://node-1:8080/health
   curl http://node-2:8080/health
   curl http://node-3:8080/health
   ```

### Debugging Tools

#### Enable Debug Logging

```bash
RUST_LOG=debug mir-runtime
```

#### Memory Profiling

```bash
valgrind --tool=massif mir-runtime
```

#### Performance Profiling

```bash
perf record -g mir-runtime
perf report
```

### Health Checks

#### HTTP Health Endpoints

```rust
// Health check endpoint
app.get("/health", |_req| async {
    let health = runtime.get_health().await;
    if health.is_healthy() {
        Ok(Response::new(200, "OK"))
    } else {
        Ok(Response::new(503, "Service Unavailable"))
    }
});

// Readiness check endpoint
app.get("/ready", |_req| async {
    let ready = runtime.is_ready().await;
    if ready {
        Ok(Response::new(200, "Ready"))
    } else {
        Ok(Response::new(503, "Not Ready"))
    }
});
```

#### Kubernetes Health Checks

```yaml
livenessProbe:
  httpGet:
    path: /health
    port: 8080
  initialDelaySeconds: 30
  periodSeconds: 10
  timeoutSeconds: 5
  failureThreshold: 3

readinessProbe:
  httpGet:
    path: /ready
    port: 8080
  initialDelaySeconds: 5
  periodSeconds: 5
  timeoutSeconds: 3
  failureThreshold: 3
```

This deployment guide covers the major deployment scenarios for the MIR Runtime. Choose the appropriate deployment method based on your infrastructure requirements and operational constraints.