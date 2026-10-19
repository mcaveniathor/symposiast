# Action integration fixture

`stucco-analysis.json` is a frozen, imported analysis for Stucco 0.2.1. Its citations refer to the normal collector's commit-pinned guide sources. The action check collects the release, imports this analysis, compiles the recipe, packages the portable bundle, uploads an artifact, and checks recipient installation. No provider response or live authentication is involved.

The fixture's HTML assertions run separately in `check_archive.py`; the action's `compiled` status itself records Cargo compilation only. Changes to evidence selection can require an explicit fixture update. This small fixture is not a replacement for the fuller [Stucco plugin](../../plugins/stucco/README.md).
