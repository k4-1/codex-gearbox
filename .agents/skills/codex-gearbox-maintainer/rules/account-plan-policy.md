# Account, availability, and effort policy

## Why it matters

The free/subscribed/API-key distinction is a product and cost boundary, not a
hint. Policy must remain predictable when account data or model catalogs are
incomplete.

## Rules

- Free accounts never invoke the Luna judge.
- API-key judging stays opt-in and never assumes ChatGPT subscription access.
- Subscribed judging requires an enabled judge, healthy usage band, an available
  judge model, and an ambiguous or near-threshold deterministic route.
- Never select a model absent from the live catalog when the catalog is known.
- Apply conservation and critical rate bands before optional judge work.
- Apply user effort caps, but never let a cap remove the high-risk minimum.
- High-risk work must retain the strongest available safe model/effort floor.
- If account/model/rate data is unavailable, use the deterministic route and
  make the fallback observable without logging prompt text.

## Tests

Every policy change should cover free, subscribed, API-key, unavailable-model,
rate-conservation, judge-failure, and high-risk floor behavior as applicable.
