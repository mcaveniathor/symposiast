# Action integration fixture

`stucco-analysis.json` contains the fixed findings and recipe for the Stucco 0.3.0 action check. Its citations refer only to the README and manifest in the checksum-verified published crate, so an unavailable optional repository snapshot does not invalidate the fixture.

`stucco-adapter.py` is a deterministic custom adapter that reads a version 2 job on stdin and returns that fixed guidance with complete example coverage on stdout. It checks the release identity and cited source hashes against the checked-in bundle, marks the cited README example as adapted, and explicitly skips other examples because this fixture covers only semantic page rendering. It derives coverage IDs from the actual inventory, which can vary with optional repository evidence. The JSON file alone is a template, not a complete analysis import for an inventoried bundle.

The action check collects the release, invokes this fixture adapter, compiles the recipe, packages the portable bundle, uploads an artifact, and checks recipient installation. No provider response or live authentication is involved. To exercise the same generation path locally:

```sh
cargo run --locked --bin symposiast -- build stucco --version 0.3.0 \
  --agent custom --agent-executable ./examples/ci/stucco-adapter.py \
  --output /tmp/stucco-action-check
```

Use a new output directory. The adapter requires Python 3 and rejects other releases or changed cited evidence. Run its offline regression tests with `python3 -m unittest discover -s .github/action -p 'test_*.py' -v`.

The fixture's HTML assertions run separately in `check_archive.py`; the action's `compiled` status itself records Cargo compilation only. Changes to evidence selection can require an explicit fixture update. This small fixture is not a replacement for the fuller [Stucco plugin](../../plugins/stucco/README.md).
