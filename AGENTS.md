# uitslagavond-tv

## Merge rule

Agents may merge their own PRs into `main` freely once required checks pass (owner decision, 2026-11-20).

## Deployment

The server and the Renderers run as plain programs on studio machines at a broadcaster, on the studio network, not in containers: the server reads a gamepad and the Renderers need the GPU and HDMI outputs directly.

## Agent skills

### Issue tracker

Issues live in the GitHub Issues of this repository, used through the `gh` CLI. See `docs/agents/issue-tracker.md`.

### Triage labels

The five default triage labels: `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: one `GLOSSARY.md` and `docs/adr/` at the repo root. See `docs/agents/domain.md`.

## Design

- `PRODUCT.md`: product context for design work (users, laptop setup, fastest actions).
- `DESIGN.md`: the visual system of the Regie dashboard and `/tablet`, design **DaVinci Deck** (clean-slate rebuild in `dashboard/src/deck.css`, replacing the legacy "Studio" / "Omroepsoftware" / "Mengpaneel" CSS layers).
