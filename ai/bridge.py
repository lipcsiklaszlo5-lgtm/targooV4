import json
import os
import psutil
import numpy as np
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
from sentence_transformers import SentenceTransformer, util
from typing import Optional, List

app = FastAPI(title="Targoo V2 AI Bridge")

# Global model loading
print("Loading sentence-transformer model...")
model = SentenceTransformer('paraphrase-multilingual-MiniLM-L12-v2')

# Global Dictionary Index
DICTIONARY_PATH = "data/dictionary.json"
dictionary_data = []
dictionary_embeddings = None

def load_dictionary():
    global dictionary_data, dictionary_embeddings
    if not os.path.exists(DICTIONARY_PATH):
        print(f"Warning: {DICTIONARY_PATH} not found.")
        return

    with open(DICTIONARY_PATH, "r", encoding="utf-8") as f:
        dictionary_data = json.load(f)

    # Flatten keywords for embedding
    all_keywords = []
    for entry in dictionary_data:
        # We use the first keyword as the primary representative for the entry
        # Or better, we could average them, but for now, let's just take the list
        all_keywords.append(" ".join(entry["keywords"]))

    print(f"Encoding {len(all_keywords)} dictionary entries...")
    dictionary_embeddings = model.encode(all_keywords, convert_to_tensor=True)

load_dictionary()

class ClassifyRequest(BaseModel):
    header: str
    value: str
    industry: Optional[str] = "General"
    threshold: float = 0.7

class MappingResponse(BaseModel):
    canonical_unit: str
    ghg_category: str
    scope3_id: Optional[int]
    confidence: float

@app.post("/classify", response_model=Optional[MappingResponse])
async def classify(req: ClassifyRequest):
    if dictionary_embeddings is None:
        raise HTTPException(status_code=503, detail="Dictionary index not loaded")

    # Encode input header
    query_embedding = model.encode(req.header, convert_to_tensor=True)
    
    # Compute cosine similarities
    cos_scores = util.cos_sim(query_embedding, dictionary_embeddings)[0]
    best_idx = int(np.argmax(cos_scores.cpu()))
    best_score = float(cos_scores[best_idx])

    if best_score >= req.threshold:
        match = dictionary_data[best_idx]
        return MappingResponse(
            canonical_unit=match["canonical_unit"],
            ghg_category=match["ghg_category"],
            scope3_id=match["scope3_id"],
            confidence=best_score
        )
    
    return None

@app.get("/health")
async def health():
    process = psutil.Process(os.getpid())
    return {
        "status": "ready",
        "model": "paraphrase-multilingual-MiniLM-L12-v2",
        "ram_usage_mb": process.memory_info().rss / 1024 / 1024,
        "index_size": len(dictionary_data) if dictionary_data else 0
    }

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=9000)
