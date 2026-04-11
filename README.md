# TARGOO V2 - Industrial ESG Data Refinery
## Immutable, Audit-Ready Carbon Accounting Engine

TARGOO V2 is a high-performance, deterministic engine designed for industrial-scale ESG data normalization and GHG Protocol-compliant carbon accounting. It ensures full traceability and data integrity through SHA-256 chaining and rigorous validation guards.

---

### 🏗 Architecture

The system is built on a robust three-tier architecture:

1.  **Frontend (Next.js / TypeScript / Tailwind):**
    *   A modern, professional "Big4-ready" dashboard for data ingestion and metadata injection.
    *   Features real-time processing feedback, interactive dropzones, and localized report generation (HU, EN, DE).

2.  **Backend (Rust / Axum):**
    *   The core deterministic processing engine.
    *   Implements **Range Guards** to prevent anomalous data entry, **SHA-256 chaining** for an immutable audit trail, and high-speed aggregation for Scopes 1, 2, and 3.
    *   Exposes a high-performance REST API for upload, execution, and status polling.

3.  **AI Bridge (Python / FastAPI / Gemini):**
    *   A semantic **Triage Motor** responsible for Scope 3 resolution.
    *   Utilizes advanced LLM capabilities to classify unstructured spend data into GHG Protocol categories with confidence scoring.

---

### 📦 The Fritz Package

Upon successful processing, the engine generates "The Fritz Package" — a ZIP archive containing 7 essential files for a complete ESG audit:

1.  **00_Manifest.json:** Contains SHA-256 hashes of all files in the package to ensure immutability and verify data integrity.
2.  **01_GHG_Inventar_Zusammenfassung.xlsx:** A high-level executive summary of the total carbon footprint (Scopes 1, 2, and 3).
3.  **02_Scope_Aufschluesselung.xlsx:** Detailed breakdown of emissions by scope, category, and physical/spend-based metrics.
4.  **03_Audit_Trail_Master.xlsx:** The complete processing log, showing every transformation, unit conversion, and the underlying hash chain.
5.  **04_Quarantaene_Log.xlsx:** Records all data points that failed validation or required manual review, ensuring no data is silently dropped.
6.  **05_Emissionsfaktoren_Referenz:** A reference table of all emission factors and GWP100 values used during calculations.
7.  **06_Narrative_Bericht.md:** An AI-generated qualitative analysis of the inventory, highlighting hotspots and reduction opportunities.

---

### 🛠 Installation & Setup

To run the full stack, you need to start all three components:

#### 1. AI Bridge (Python)
```bash
cd ai
pip install -r requirements.txt
python bridge.py
```
*(Runs on http://127.0.0.1:5000)*

#### 2. Backend (Rust)
```bash
# Ensure your .env file has the required GEMINI_API_KEY
cargo run
```
*(Runs on http://127.0.0.1:8080)*

#### 3. Frontend (Next.js)
```bash
cd frontend
npm install
npm run dev
```
*(Runs on http://127.0.0.1:3000)*

---

### 🛡 Security & Compliance
*   **Deterministic Engine:** The same input always yields the same hashed output.
*   **Range Guards:** Outlier detection prevents multi-ton errors from simple typo/unit mistakes.
*   **Audit-Ready:** Designed to withstand rigorous external auditor scrutiny with 100% transparent calculation paths.
