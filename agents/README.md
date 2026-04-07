# Pipecheck Agent System

## Overview
This directory contains specialized AI agents for developing the pipecheck CI/CD Pipeline Auditor.

## Active Agents

### 1. ArchitectAgent
**Role:** System Architecture & Design  
**Focus:** Project structure, module design, technical decisions  
**File:** `architect_agent.json`

### 2. CoreDeveloperAgent
**Role:** Core Rust Implementation  
**Focus:** Parsers, auditors, core logic  
**File:** `core_developer_agent.json`

### 3. WASMAgent
**Role:** WebAssembly Integration  
**Focus:** WASM compilation, npm package, browser support  
**File:** `wasm_agent.json`

### 4. TestingAgent
**Role:** Quality Assurance & Testing  
**Focus:** Test suites, fixtures, benchmarks  
**File:** `testing_agent.json`

### 5. DevOpsAgent
**Role:** CI/CD, Release & Distribution  
**Focus:** GitHub Actions, releases, cross-platform builds  
**File:** `devops_agent.json`

### 6. DocumentationAgent
**Role:** Documentation & Developer Experience  
**Focus:** README, guides, API docs, examples  
**File:** `documentation_agent.json`

### 7. ProjectManagerAgent
**Role:** Project Coordination & Planning  
**Focus:** Timeline, milestones, coordination  
**File:** `project_manager_agent.json`

## Current Phase: Foundation (Week 1-2)

### Immediate Tasks
1. ✅ Agent system created
2. ⏳ Initialize Cargo project
3. ⏳ Setup project structure
4. ⏳ Implement basic types and traits
5. ⏳ Create GitHub Actions parser
6. ⏳ Build syntax validator

## Usage
Each agent JSON file contains:
- Responsibilities and scope
- Implementation details
- Commands to execute
- Key decisions and patterns

Refer to the appropriate agent when working on specific aspects of the project.
