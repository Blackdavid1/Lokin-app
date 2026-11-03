# reputation-registry — On-Chain MoiScore

Purpose: On-chain reputation scoring for Loki-app participants. Maintains per-user MoiScore (0–1000), tier (Bronze→Diamond), activity history, streaks, and completion/default counts. Tier determines benefits such as circle size limits, contribution caps, and collateral requirements. Circle payout events feed score updates via `record_on_time_payment`, `record_circle_completion`, `record_default`, and `apply_inactivity_decay`.

## Entry points

- `init(admin)` — initialize with admin address
- `record_activity(user, activity_type, score_impact)` — record a generic activity (admin/organizer)
- `get_score(user)` — Result<MoiScore> current score + tier + stats
- `get_history(user, page, page_size)` — Vec<Activity> paginated activity history
- `calc_collateral(member)` — u32 derived collateral amount
- `calc_max_size(member)` — u32 max circle size for member
- `calc_max_contrib(member)` — i128 max contribution for member
- `get_moi_score(member)` — u32 current score
- `get_moi_tier(member)` — u32 tier (0=Bronze … 4=Diamond)

Score-modifying (auth: member.require_auth(); gated by `when_not_paused`):
- `record_on_time_payment(member, circle_id, amount, round)` — Result<u32> new score
- `record_circle_completion(member)` — Result<u32> new score
- `record_default(member)` — Result<u32> new score
- `apply_inactivity_decay(member, days_inactive)` — Result<u32> new score

Pause:
- `pause_registry(admin)` / `unpause_registry(admin)` — emergency stop

## Storage keys (DataKey)

| Key | Meaning |
|-----|---------|
| `Admin` | registry admin |
| `Score(Address)` | per-user MoiScore |
| `UserActivityPage(Address, u32)` | paginated activity history |
| `UserActivityCount(Address)` | activity count per user |
| `Streak(Address, Address)` | streak state |
| `Completions(Address)` | completion count |
| `MemberScore(Address)` | member score index |
| `Defaults(Address)` | default count |
| `MemberLog(Address)` | member action log |
| `LastRound(Address, Address)` | last round per (member, circle) |

## Events

| Event | Topics | Meaning |
|-------|--------|---------|
| `ActivityRecorded { user, activity_type, score_impact, new_score }` | — | activity recorded |
| `ScoreUpdated { user, old_score, new_score, tier }` | — | score changed |

## Activity types

| Value | Constant | Meaning |
|-------|----------|---------|
| 0 | `ACTIVITY_JOIN` | joined a circle |
| 1 | `ACTIVITY_CONTRIBUTE` | contributed on time |
| 2 | `ACTIVITY_COMPLETE` | completed a circle |
| 3 | `ACTIVITY_DEFAULT` | defaulted on a circle |
| 4 | `ACTIVITY_PAYOUT_RECEIVED` | received a payout |

## Tiers

| Value | Constant | Range |
|-------|----------|-------|
| 0 | `TIER_BRONZE` | base |
| 1 | `TIER_SILVER` | — |
| 2 | `TIER_GOLD` | — |
| 3 | `TIER_PLATINUM` | governance voting |
| 4 | `TIER_DIAMOND` | zero collateral |

## Invariants
- Score-modifying functions require the member's auth (`member.require_auth()`).
- All score writes are gated by `when_not_paused` (`ContractPaused`).
- Invalid activity type or score impact returns typed errors (`InvalidActivityType`, `InvalidScoreImpact`).
- Arithmetic overflow returns `Overflow`, not a panic.
- Score not found returns `ScoreNotFound`.

## Test command
```bash
# From repo root
cargo test -p reputation-registry
cargo test -p reputation-registry --doc
# Full workspace
cargo test --workspace
```
