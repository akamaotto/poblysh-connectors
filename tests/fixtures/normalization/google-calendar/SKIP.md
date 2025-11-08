# Google Calendar Normalization

## Rationale
The current connector emits stubbed `calendar_event_*` kinds, but the
normalization harness needs real webhook payload samples before we can lock the
behavior with fixtures.

## Plan
Collect representative webhook payloads for `calendar_event_updated` and
`calendar_event_deleted`, expose a helper that maps resource state → SignalKind,
and add fixtures once the connector consumes the shared logic.
