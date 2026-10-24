"""Save experiment results as non-overwriting JSON files under results/."""
import json
from datetime import datetime, timezone
from pathlib import Path
from uuid import uuid4

ROOT = Path(__file__).resolve().parent


def save_result(experiment, payload, category=None):
    now = datetime.now(timezone.utc)
    folder = ROOT / "results" / experiment
    if category:
        folder /= category
    folder.mkdir(parents=True, exist_ok=True)
    filename = f"{now.strftime('%Y%m%dT%H%M%S_%fZ')}_{uuid4().hex[:8]}.json"
    record = {"schema_version": 1, "experiment": experiment,
              "recorded_at_utc": now.isoformat(), **payload}
    path = folder / filename
    with path.open("x", encoding="utf-8") as stream:
        json.dump(record, stream, indent=2, sort_keys=True)
        stream.write("\n")
    print(f"\nSaved JSON: {path.relative_to(ROOT)}")
    return path
