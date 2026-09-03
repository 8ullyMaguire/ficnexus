"""Vowpal Wabbit contextual-bandit sidecar recipe.

    pip install vowpalwabbit flask
    python vw_bandit_recipe.py
"""
import os

from flask import Flask, jsonify, request

try:
    from vowpalwabbit import pyvw
except ImportError:  # pragma: no cover
    pyvw = None

app = Flask(__name__)
MODEL = None


def _vw() -> str:
    # Contextual bandit: shared features = user tag vector (simplified here
    # to a dummy); actions = candidate work ids passed per request.
    return "--cb_explore 10 --epsilon 0.1 --quiet"


@app.post("/score")
def score():
    global MODEL
    if MODEL is None:
        return jsonify({"recs": []}), 503
    payload = request.get_json(force=True)
    # Sidecar keeps its own action list from /train; score returns pmf.
    preds = MODEL.predict("shared |U user:%d" % (payload.get("user_id") or 0))
    recs = [{"work_id": a, "score": float(p), "reason": "vw-bandit"}
            for a, p in zip(ACTIONS, preds)]
    return jsonify({"recs": recs})


@app.post("/train")
def train():
    global MODEL
    if pyvw is None:
        return jsonify({"ok": False, "error": "vowpalwabbit not installed"}), 500
    MODEL = pyvw.Workspace(vw_learn=_vw())
    return jsonify({"ok": True})


# Actions are seeded from the last /train call in a real deployment; the
# FicHub ranker passes the candidate pool via the payload in the full
# integration. Kept minimal here for the recipe.
ACTIONS = []


@app.post("/actions")
def actions():
    global ACTIONS
    ACTIONS = request.get_json(force=True).get("work_ids", [])
    return jsonify({"ok": True, "actions": len(ACTIONS)})


if __name__ == "__main__":
    app.run(host="0.0.0.0", port=int(os.environ.get("PORT", 8302)))
