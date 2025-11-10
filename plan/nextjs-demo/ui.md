connectors/plan/nextjs-demo/ui.md
# Next.js Demo UX Plan — Black & White Minimalist UI

Purpose:
- A clean, black and white mock-only UX that demonstrates the Connectors story.
- Fast to scan. Brevity and elegance over exposition.
- Built with:
  - Next.js App Router (Next 16+ semantics)
  - Tailwind CSS
  - shadcn/ui primitives
  - Native Next.js components (links, navigation, route handlers)

## Black & White Color System

Primary Palette:
- Background: pure white (`bg-white`)
- Primary text: black (`text-black`)
- Secondary text: gray-800 (`text-gray-800`)
- Borders: gray-200 (`border-gray-200`)
- Dividers: gray-100 (`bg-gray-100`)

Interactive Elements:
- Buttons: black background (`bg-black text-white hover:bg-gray-800`)
- Input borders: black (`border-black focus:border-black`)
- Focus rings: black (`focus:ring-black`)

Status Colors:
- Error text: red-500 (`text-red-500`)
- Error borders: red-300 (`border-red-300`)
- Success badges: green-600 (`text-green-600 bg-green-50 border-green-200`)
- Warning badges: yellow-600 (`text-yellow-600 bg-yellow-50 border-yellow-200`)

Grayscale Hierarchy:
- Text primary: black (`text-black`)
- Text secondary: gray-800 (`text-gray-800`)
- Text muted: gray-600 (`text-gray-600`)
- Borders light: gray-200 (`border-gray-200`)
- Backgrounds: white (`bg-white`) with gray-50 (`bg-gray-50`) accents
- UI elements: gray-800/gray-900 range

General:
- One clear headline per page.
- One clear primary action per page.
- Short labels, short descriptions.
- Use whitespace and type, not decoration.
- Inline teaching kept minimal; deeper context goes into collapsible sections.

Global:
- Persistent top banner/label: “Mock UX Demo — No real data or connections.”
- Simple top nav (after mock sign-in):
  - Tenant
  - Integrations
  - Signals
- Deterministic mock behavior; reloads feel stable.
- Responsive enough; desktop-first.

---

## Pages

### `/` — Landing & Mock Sign-In

Layout:
- Centered shadcn `Card` on white background.

Content:
- Title: “Poblysh Connectors UX Demo”
- Subtitle (1 line): “Click through the mock integration flow. No real calls.”
- Badge: “Mock-only”
- Email `Input`
- Primary `Button`: “Continue”

Behavior:
- On submit:
  - Create mock user in client state.
  - Navigate to `/tenant`.

Details (collapsible):
- Implementation notes: mock-only auth, no passwords.
- Mapping: where real SSO/session would exist.

---

### `/tenant` — Tenant Setup

Layout:
- Centered `Card`.

States:
- No tenant:
  - Title: “Set up your tenant”
  - Input: “Company name”
  - Primary `Button`: “Create tenant”
- With tenant:
  - Show:
    - “Poblysh Tenant ID”
    - “Connectors Tenant ID (X-Tenant-Id)”
  - Primary `Button`: “Continue to Integrations”

Tone:
- Plain labels, no long paragraphs.

Details (collapsible):
- One short explanation of how `X-Tenant-Id` is derived and used.
- Clarify: frontend does not set it directly in production.

---

### `/integrations` — Integrations Hub

Layout:
- Header row:
  - Title: “Integrations”
  - Small text: “Tenant: {name}”
- Grid or vertical stack of shadcn `Card`s.

Cards (examples):
- GitHub
- Zoho Cliq

Each card:
- Provider name + monochrome icon.
- Status badge:
  - "Connected" (`bg-black text-white`)
  - "Not connected" (`bg-white border border-black text-black`)
  - Error (`bg-red-50 text-red-600 border border-red-200`)
- Primary `Button`: always black (`bg-black text-white hover:bg-gray-800`)
  - "Connect" if not connected.
  - "Disconnect" if connected.
- Secondary `Button`:
  - "Details" or "Scan for signals" when relevant (`border border-black text-black hover:bg-gray-50`).

Connect flow (mock):
- On “Connect”:
  - Minimal shadcn `Dialog`:
    - One-line consent text.
    - “Cancel” / “Authorize”
  - On “Authorize”:
    - Mark as connected.
    - Toast: “GitHub connected (mock).”

Details (collapsible):
- Note where real OAuth + Connectors endpoints would plug in.

---

### `/signals` — Signals List

Entry:
- From nav, or from “Scan for signals” on `/integrations`.

Layout:
- Header:
  - Title: “Signals”
  - Optional short caption: “Mock signals from connected integrations.”
- Controls:
  - Provider filter (monochrome `Badge`/`Select` with black borders).
  - Simple search `Input` with black borders and focus states.
  - "Scan" `Button`:
    - Always black (`bg-black text-white hover:bg-gray-800`)
    - Disabled state: `bg-gray-100 text-gray-400 border-gray-200`

States:
- No connections:
  - Simple empty state:
    - Text: “Connect an integration to see signals.”
    - Button: “Go to Integrations”
- Connected, no scan:
  - Text: “Run a mock scan to generate signals.”
  - Button: “Scan”
- With signals:
  - shadcn `Table` or list:
    - Columns: Provider, Kind, Title, Occurred at.
    - Rows clickable → `/signals/[id]`.

Scan behavior (mock):
- Single short loading skeleton.
- Deterministic generated signals.
- Toast: “Scan complete (mock).”

Details (collapsible):
- How this mirrors `GET /signals` with `X-Tenant-Id`.

---

### `/signals/[id]` — Signal Detail & Grounding

Layout:
- Back link: “← Signals”
- Title from signal.
- Provider badge (monochrome).

Sections:
1) Summary
- Kind
- Provider
- Occurred at
- Key metadata (repo/channel/etc.)

2) Raw data
- Small shadcn `Collapsible`:
  - Label: “View raw payload”
  - Inside: formatted JSON-like block (monochrome).

3) Grounding
- shadcn `Card` with black border (`border-black`):
  - Title: "Ground this signal"
  - Short line: "Combine related activity into one view."
  - Primary `Button`: "Ground" (`bg-black text-white hover:bg-gray-800`)

On “Ground”:
- Generate mock grounded signal:
  - Overall score (0–100)
  - Per-dimension scores (Relevance, Impact, etc.)
  - Evidence list:
    - GitHub items
    - Zoho Cliq items (if connected)
    - Optional “web snippet”
- Display:
  - Score as monochrome badge or progress bar (`bg-gray-200` with black fill).
  - Evidence as simple list with black borders.
  - One-line explanation in `text-gray-600`.

Final success:
- If showing "grounded" completion, use a single green badge:
  - `bg-green-50 text-green-600 border border-green-200`
  - e.g., "Grounded" or "Ready".
- No other green usage in the interface.

Details (collapsible):
- How real grounding would call APIs and aggregate sources.

---

## Components & Patterns

### Black & White Component System

Primary Elements:
- **Buttons**: Always `bg-black text-white hover:bg-gray-800`
- **Inputs**: `border-black focus:border-black focus:ring-black`
- **Cards**: `border-black` or `border-gray-200`
- **Badges**: Monochrome with black text/white backgrounds or white text/black backgrounds

Use shadcn/ui:
- Layout: `Container` / `Card` / `Separator` with black borders
- Inputs: `Input`, `Label` with black focus states
- Actions: `Button` (always black), `Dialog` with black borders
- Feedback: `Badge` (monochrome), `Toast`, `Skeleton`, `Collapsible`

Use native Next.js:
- `app/` router.
- `Link` for navigation (black text with underline on hover).
- Server components for layout + static.
- Client components for interactive state.

## Typography System

- Headings: `text-black font-semibold`
- Body text: `text-gray-800`
- Muted text: `text-gray-600`
- Error text: `text-red-500`

## Interaction States

- Hover: Slightly lighter gray (`hover:bg-gray-800` for buttons)
- Focus: Black outline (`focus:ring-black`)
- Disabled: Light gray with muted text (`bg-gray-100 text-gray-400`)
- Loading: Skeleton with gray-200 background

Copy style:
- Short.
- Concrete.
- No marketing fluff.
- Teaching content hidden behind collapsibles or tooltips.

Accessibility:
- Proper semantics: Use semantic HTML5 elements (`<button>`, `<input>`, `<main>`, `<nav>`, etc.)
- Clear labels: All interactive elements must have accessible labels and descriptions
- Color is a hint, not the only signal: Ensure all information is conveyed through text, icons, or other means
- High contrast maintained throughout (black/white/gray only)
- Keyboard navigation: All interactive elements must be keyboard accessible
- Screen reader support: Use ARIA labels and roles appropriately
- Focus management: Clear focus indicators (`focus:ring-2 focus:ring-black`) and logical tab order
- Text alternatives: All meaningful images and icons must have alt text or ARIA labels
- Error announcements: Form errors should be announced to screen readers

## Specific Accessibility Requirements

Interactive Elements:
- Buttons: Use `<button>` element with accessible inner text
- Links: Use `<a>` element with descriptive text
- Forms: Use `<label>` elements properly associated with inputs
- Dialogs: Implement proper focus trapping and escape key handling

Screen Reader Support:
- Page structure: Use proper heading hierarchy (`h1`, `h2`, etc.)
- Navigation: Use `<nav>` with proper ARIA landmarks
- Main content: Use `<main>` with appropriate landmarks
- Status updates: Use `aria-live` regions for dynamic content

Keyboard Navigation:
- Tab order: Logical and intuitive navigation flow
- Focus indicators: Visible black focus rings on all interactive elements
- Skip links: Provide skip-to-content links for keyboard users
- Modal focus: Proper focus management in dialogs and modals

## Implementation Notes

All interactive elements must follow the black and white system:
- No blue, green, yellow, or other accent colors except:
  - Red for error states only
  - Single green badge for final "grounded" success state
- Consistent use of black backgrounds for primary actions
- White backgrounds with black borders for secondary actions
- Gray scale hierarchy for all other elements

End.