# Phase 2 Architecture Progression

## Current State (Pre-Phase 2)
```
┌─────────────┐     ┌──────────────┐     ┌────────────────┐
│   Frontend  │────▶│ API Gateway  │────▶│    Monolith    │
│  (Port 5173)│     │ (Port 8089)  │     │  (Port 3000)   │
└─────────────┘     └──────────────┘     │                │
                                         │ - Auth         │
                                         │ - Samples      │
                                         │ - Templates    │
                                         │ - Storage      │
                                         │ - Sequencing   │
                                         │ - Notifications│
                                         │ - RAG          │
                                         │ - Reports      │
                                         │ - Dashboard    │
                                         └────────────────┘
```

## Week 1: Safe Services Enabled
```
┌─────────────┐     ┌──────────────┐     ┌────────────────┐
│   Frontend  │────▶│ API Gateway  │────▶│    Monolith    │
│  (Port 5173)│     │ (Port 8089)  │     │  (Port 3000)   │
└─────────────┘     │              │     │                │
                    │ Feature Flags│     │ - Auth         │
                    │ Enabled:     │     │ - Samples      │
                    │ ✅ Notif     │     │ - Templates    │
                    │ ✅ Storage   │     │ - Sequencing   │
                    │ ✅ RAG       │     │ - Reports      │
                    └──────┬───────┘     │ - Dashboard    │
                           │             └────────────────┘
                           │
        ┌──────────────────┼──────────────────┐
        │                  │                  │
        ▼                  ▼                  ▼
┌───────────────┐  ┌───────────────┐  ┌───────────────┐
│ Notification  │  │   Storage     │  │     RAG       │
│   Service     │  │   Service     │  │   Service     │
│ (Port 3016)   │  │ (Port 3014)   │  │ (Port 3019)   │
└───────────────┘  └───────────────┘  └───────────────┘
```

## Week 2: Template Service Added
```
┌─────────────┐     ┌──────────────┐     ┌────────────────┐
│   Frontend  │────▶│ API Gateway  │────▶│    Monolith    │
│  (Port 5173)│     │ (Port 8089)  │     │  (Port 3000)   │
└─────────────┘     │              │     │                │
                    │ Feature Flags│     │ - Auth         │
                    │ ✅ Notif     │     │ - Samples      │
                    │ ✅ Storage   │     │ - Sequencing   │
                    │ ✅ RAG       │     │ - Reports      │
                    │ ✅ Templates │     │ - Dashboard    │
                    └──────┬───────┘     └────────────────┘
                           │
        ┌──────────────────┼─────────────────────┐
        │                  │                     │
        ▼                  ▼                     ▼
┌───────────────┐  ┌───────────────┐    ┌───────────────┐
│ Notification  │  │   Storage     │    │   Template    │
│   Service     │  │   Service     │    │   Service     │
│ (Port 3016)   │  │ (Port 3014)   │    │ (Port 3013)   │
└───────────────┘  └───────────────┘    └───────────────┘
                           │
                   ┌───────────────┐
                   │     RAG       │
                   │   Service     │
                   │ (Port 3019)   │
                   └───────────────┘
```

## Week 3: Auth Service (HIGH RISK) 🔐
```
┌─────────────┐     ┌──────────────┐     ┌────────────────┐
│   Frontend  │────▶│ API Gateway  │────▶│    Monolith    │
│  (Port 5173)│     │ (Port 8089)  │     │  (Port 3000)   │
└─────────────┘     │              │     │                │
                    │ Feature Flags│     │ - Samples      │
                    │ ✅ Auth 🔐   │     │ - Sequencing   │
                    │ ✅ Templates │     │ - Reports      │
                    │ ✅ Storage   │     │ - Dashboard    │
                    │ ✅ Notif     │     └────────────────┘
                    │ ✅ RAG       │
                    └──────┬───────┘
                           │
        ┌──────────────────┼─────────────────────┐
        │                  │                     │
        ▼                  ▼                     ▼
┌───────────────┐  ┌───────────────┐    ┌───────────────┐
│     Auth      │  │   Storage     │    │   Template    │
│   Service 🔐  │  │   Service     │    │   Service     │
│ (Port 3010)   │  │ (Port 3014)   │    │ (Port 3013)   │
└───────────────┘  └───────────────┘    └───────────────┘
        │                  │                     │
┌───────────────┐  ┌───────────────┐    ┌───────────────┐
│ Notification  │  │     RAG       │    │               │
│   Service     │  │ (Port 3019)   │    │    Future     │
│ (Port 3016)   │  │   Service     │    │   Services    │
└───────────────┘  └───────────────┘    └───────────────┘
```

## Week 6: Final State (Monolith Removed) 🎉
```
┌─────────────┐     ┌──────────────┐
│   Frontend  │────▶│ API Gateway  │
│  (Port 5173)│     │ (Port 8089)  │
└─────────────┘     │              │
                    │ All Services │
                    │   Enabled    │
                    └──────┬───────┘
                           │
    ┌──────────────────────┼──────────────────────┐
    │                      │                      │
    ▼                      ▼                      ▼
┌─────────┐         ┌─────────┐           ┌─────────┐
│  Auth   │         │ Sample  │           │Template │
│Service  │         │Service  │           │Service  │
│  3010   │         │  3011   │           │  3013   │
└─────────┘         └─────────┘           └─────────┘
    │                      │                      │
┌─────────┐         ┌─────────┐           ┌─────────┐
│Storage  │         │Sequenc. │           │  RAG    │
│Service  │         │Service  │           │Service  │
│  3014   │         │  3012   │           │  3019   │
└─────────┘         └─────────┘           └─────────┘
    │                      │                      │
┌─────────┐         ┌─────────┐           ┌─────────┐
│ Notif.  │         │Dashboard│           │Reports  │
│Service  │         │Service  │           │Service  │
│  3016   │         │  (New)  │           │  (New)  │
└─────────┘         └─────────┘           └─────────┘

                    🚫 Monolith Removed 🚫
```

## Service Port Reference

| Service | Port | Week | Risk Level |
|---------|------|------|------------|
| API Gateway | 8089 | Pre-existing | - |
| Notification | 3016 | Week 1 | Low |
| Storage | 3014 | Week 1 | Low |
| RAG | 3019 | Week 1 | Low |
| Template | 3013 | Week 2 | Medium |
| Auth | 3010 | Week 3 | **HIGH** |
| Sample | 3011 | Week 4 | High |
| Sequencing | 3012 | Week 4 | High |
| Dashboard | TBD | Week 5 | Low |
| Reports | TBD | Week 5 | Low |
| Spreadsheet | TBD | Week 5 | Low |

## Migration Progress Indicator

```
Week 1: [███░░░░░░░] 30% - Safe Services
Week 2: [█████░░░░░] 50% - Template Added
Week 3: [███████░░░] 70% - Auth Migrated
Week 4: [████████░░] 80% - Core Services
Week 5: [█████████░] 90% - New Services
Week 6: [██████████] 100% - Migration Complete! 🎉
```