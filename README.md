<div align="center">
    <img src="https://i.imgur.com/qnx9diF.png" alt="CHATq Logo"/>
    <h1>CHATq</h1>
    <h3>Single-Source, Multi-Audience Message Exchange Archive & Query System</h3>
</div>

---

**CHATq** is a concurrent, gRPC-based message archival and querying backend. It provides a robust engine for routing, persisting, and querying messages mapped dynamically between **Sources** (senders) and **Audiences** (recipients/groups).

## Tech Stack

- **Rust**: Core backend (`chatq-server`).
- **gRPC / Protocol Buffers**: Used for all inter-service communication (`tonic`, `prost`).
- **Tokio**: Asynchronous runtime handling high-throughput streams and real-time broadcasting event channels.
- **PostgreSQL**: Primary data persistence store.
- **SQLx**: Compile-time verified asynchronous SQL execution and schema migrations.

## Workspace Structure

This repo is structured as a multi-crate workspace:

- **`chatq-server`**: The core logic engine. Handles DB persistence, snapshot pagination logic, and houses the gRPC server. Implements a hexagonal architecture via ports/interfaces (`src/ports.rs`).
- **`chatq-types`**: Shared types, Protobuf definitions, and generated gRPC bindings.
- **`chatq-gateway`** *(Planned)*: API gateway / edge service for REST and WebSocket ingress.
- **`chatq-webui`** *(Planned)*: Administrative and exploration web frontend.

## Domain & Architecture

### System Architecture & Data Flow

`chatq-server` manages message delivery and historical indexing across an append-only event stream, routing events from actors to target audiences within isolated scopes.

#### Core Data Entities

* **Entity**: Any addressable actor within the system, including human users, service accounts, and automated agents.
* **Scope**: The routing and sharding boundary for message isolation, such as a channel, guild, room, or shard.
* **Event**: An atomic, immutable message envelope containing the sender ID, recipient list, scope ID, ingestion timestamp, and payload.
* **Timeline**: Monotonically ordered log sequence numbers or timestamps ensuring total delivery and ingestion order.

#### Service Design

Built on Hexagonal Architecture (Ports and Adapters), the service keeps core domain logic isolated from external infrastructure:

* **Inbound Ports**: Entry points exposing high-throughput gRPC endpoints for message ingestion, real-time streaming, and history queries.
* **Domain Layer**: Enforces routing rules, recipient resolution, and filter tree evaluations.
* **Outbound Ports**: Pluggable storage and streaming interfaces (`MessageRepo`, `MessageEventChannel`) that decouple domain behavior from possible underlying persistence systems like PostgreSQL, ScyllaDB, Redis, or Kafka.

#### Query Engine & Retrieval

Historical message access operates on an append-only log sorted by ingestion time:

* **Filter Compilation**: An AST evaluates arbitrary boolean logic (`AND`, `OR`, `NOT`) against message attributes like sender, scope, and target recipients, compiling directly to indexed database queries.
* **Keyset Cursor Pagination**: State-preserving queries rely on composite timestamp and message ID cursors to support deterministic scrolling and stable window traversal without offset performance penalties.
* **Stateless Range Queries**: Ad-hoc fetches bounded strictly by explicit time intervals or ID ranges.
* **Audit & Point-in-Time Snapshots**: Queries can freeze an immutable historical window while snapshotting user-to-identifier mappings, ensuring name changes or profile updates do not alter historical records.

---

<details>
<summary><strong>Theoretical Formalization (Derived Specifications)</strong></summary>

> **THE FOLLOWING IS INTENDED FOR POSSIBLE DERIVED FORMALIZATION EFFORT**

#### Formal Model

The system's information flow can be formalized as a directed, attributed temporal hypergraph **ℋ = (V, E)**, defined across:

* **V**: The set of all unique entities (e.g., users, players, agents) in the ecosystem.
* **Γ**: The domain of context qualifiers (e.g., channel, server, zone, or scope identifiers).
* **T**: The continuous temporal domain (**T ⊆ ℝ**), represented via standardized timestamps.

Every transmitted message constitutes an attributed directed hyperedge **e ∈ E**:

> **e = (s, A, γ, t)**

where:

* **s ∈ V** is the **Source** vertex originating the event.
* **A ⊆ V** is the target **Audience** subset receiving the event.
* **γ ∈ Γ** is the **Context** attribute qualifying the spatial scope.
* **t ∈ T** is the **Temporal Anchor** representing ingestion or archive time.

#### Functional Invariants

`chatq-server` structurally isolates state side effects from core invariants using Hexagonal Architecture (Ports & Adapters). State transitions and query traversals execute against boundary traits (such as `MessageRepo` and `MessageEventChannel`), decoupling network protocols (gRPC) and persistence mechanisms (PostgreSQL) from domain invariants.

#### Navigating the Message History Poset

Message history forms a partially ordered set (poset) of timestamped events **(E, ≤)**, totally ordered along the ingestion timeline. Retrieval corresponds to evaluating a filter predicate **φ : E → {⊤, ⊥}** that selects an induced sub-poset:

> **E_φ = { e ∈ E | φ(e) = ⊤ }**

Because the filter AST supports universal conjunction (`AND`), disjunction (`OR`), and negation (`NOT`), the space of valid filters forms a **Boolean lattice** **(Φ, ⊓, ⊔, ¬)** under logical entailment. The query engine acts as a lattice homomorphism mapping filter operations directly to set operations over the event archive:

* **E_(φ₁ ⊓ φ₂) = E_φ₁ ∩ E_φ₂**
* **E_(φ₁ ⊔ φ₂) = E_φ₁ ∪ E_φ₂**

The retrieval subsystem supports:

* **Session-Based vs. Sessionless Queries**: Stateful cursor navigation with fixed windows versus discrete, stateless lookups.
* **Audit Snapshots**: Freezes an immutable sub-poset matching a query pattern alongside a resolved point-in-time `NameUuidMap` for consistent historical inspection.

</details>

### In-Memory Event Streaming

Real-time delivery utilizes `tokio::sync::broadcast` channels. When an ingested hyperedge commits to the persistent store, matching predicates evaluate concurrently, fanning out message payloads to active listener streams without blocking ingestion threads.

## Local Development

### Prerequisites

Ensure you have the following installed:
* [Rust toolchain](https://rustup.rs/) (1.75+)
* [Docker](https://docs.docker.com/get-docker/) & Docker Compose
* `sqlx-cli` (optional, for manual database migrations)

### 1. Database Setup

Once the given env variables are declared locally, launch the local PostgreSQL container:

```bash
docker-compose up -d database