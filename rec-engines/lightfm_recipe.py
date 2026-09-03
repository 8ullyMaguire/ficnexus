"""LightFM sidecar recipe — hybrid recommender over FicHub signals.

NOT part of the default runtime. Run as:
    pip install lightfm flask psycopg[binary] numpy
    python lightfm_recipe.py            # serves :8300 /score + /train

Protocol: POST /score {"seed","user_id","n"} -> {"recs":[{work_id,score,reason}]}
"""
import os

import numpy as np
from flask import Flask, jsonify, request

try:
    from lightfm import LightFM
    from lightfm.data import Dataset
except ImportError:  # pragma: no cover
    LightFM = None

app = Flask(__name__)
MODEL = None
ITEM_IDS = []  # work_id strings in matrix column order


@app.post("/score")
def score():
    payload = request.get_json(force=True)
    if MODEL is None:
        return jsonify({"recs": []}), 503
    n = int(payload.get("n", 20))
    # Personal: user row -> top n items. Seed-only: item similarities.
    if payload.get("user_id") is not None:
        uid = int(payload["user_id"])
        scores = MODEL.predict(uid, np.arange(len(ITEM_IDS)))
    else:
        seed = payload.get("seed")
        if seed not in ITEM_IDS:
            return jsonify({"recs": []}), 200
        sid = ITEM_IDS.index(seed)
        scores = MODEL.predict(np.arange(0), np.arange(len(ITEM_IDS)) * 0 + sid)[0]
        scores[sid] = -1
    order = np.argsort(-scores)[:n]
    recs = [
        {"work_id": ITEM_IDS[i], "score": float(scores[i]), "reason": "lightfm"}
        for i in order
        if float(scores[i]) > 0
    ]
    return jsonify({"recs": recs})


@app.post("/train")
def train():
    global MODEL, ITEM_IDS
    if LightFM is None:
        return jsonify({"ok": False, "error": "lightfm not installed"}), 500
    # In production read rec_user_signals via SQL; here a placeholder CSV.
    import csv

    rows = []
    csv_path = os.environ.get("SIGNALS_CSV", "signals.csv")
    with open(csv_path) as f:
        for r in csv.DictReader(f):
            rows.append((int(r["user_id"]), r["work_id"], float(r["signal_weight"])))
    dataset = Dataset()
    dataset.fit(users={u for u, _, _ in rows}, items={w for _, w, _ in rows})
    (inter, _) = dataset.build_interactions([(u, w, {"w": wgt}) for u, w, wgt in rows])
    MODEL = LightFM(loss="warp", no_components=32)
    MODEL.fit(inter, epochs=15, num_threads=4)
    ITEM_IDS = list(dataset.mapping()[1].keys())
    return jsonify({"ok": True, "items": len(ITEM_IDS), "interactions": len(rows)})


if __name__ == "__main__":
    app.run(host="0.0.0.0", port=int(os.environ.get("PORT", 8300)))
