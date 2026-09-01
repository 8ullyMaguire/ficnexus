"""implicit-package ALS sidecar recipe — drop-in for the Rust `mf` strategy.

    pip install implicit scipy flask psycopg[binary]
    python implicit_recipe.py
"""
import os

import numpy as np
from flask import Flask, jsonify, request

app = Flask(__name__)
MODEL = None
USER_IDS = []
ITEM_IDS = []


@app.post("/score")
def score():
    global MODEL
    if MODEL is None:
        return jsonify({"recs": []}), 503
    if request.get_json(force=True).get("user_id") is None:
        return jsonify({"recs": []}), 200  # item-to-item not served here
    uid = int(request.get_json(force=True)["user_id"])
    if uid not in USER_IDS:
        return jsonify({"recs": []}), 200
    ids, scores = MODEL.recommend(
        USER_IDS.index(uid), MODEL.user_items[USER_IDS.index(uid)], N=20
    )
    recs = [
        {"work_id": ITEM_IDS[i], "score": float(s), "reason": "implicit-als"}
        for i, s in zip(ids, scores)
    ]
    return jsonify({"recs": recs})


@app.post("/train")
def train():
    global MODEL, USER_IDS, ITEM_IDS
    from implicit.als import AlternatingLeastSquares
    from scipy.sparse import coo_matrix

    import csv

    rows = []
    with open(os.environ.get("SIGNALS_CSV", "signals.csv")) as f:
        for r in csv.DictReader(f):
            rows.append((int(r["user_id"]), r["work_id"], float(r["signal_weight"])))
    USER_IDS = sorted({u for u, _, _ in rows})
    ITEM_IDS = sorted({w for _, w, _ in rows})
    u_idx = {u: i for i, u in enumerate(USER_IDS)}
    i_idx = {w: i for i, w in enumerate(ITEM_IDS)}
    mat = coo_matrix(
        (
            [w for _, _, w in rows],
            ([u_idx[u] for u, _, _ in rows], [i_idx[w] for _, w, _ in rows]),
        ),
        shape=(len(USER_IDS), len(ITEM_IDS)),
    ).tocsr()
    MODEL = AlternatingLeastSquares(factors=64, iterations=15)
    MODEL.fit(mat)
    return jsonify({"ok": True, "users": len(USER_IDS), "items": len(ITEM_IDS)})


if __name__ == "__main__":
    app.run(host="0.0.0.0", port=int(os.environ.get("PORT", 8301)))
