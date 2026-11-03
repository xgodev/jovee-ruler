# Working agreement for this repo

JoVee Ruler: the printed rulers plus their digital, interactive version (standalone app and library
inside JoVee Rig / JoVee Studio). Design: `docs/design/README.md`.

## Every change starts from an issue (the JoVee Studio flow)

A change the owner asks for — however small, even a fix to work just delivered — first gets its GitHub
issue: search open and closed ones (`gh issue list --state all --search ...`), open one only if none covers
it (the app's issues live in `xgodev/jovee-jam`, where the ruler runs). Then: clone in `.solvers/issue-N`,
branch `{type}/issue-N` from `main`, merge into `main` from the clone and push, `gh issue comment` after each
push, and the validation checklist on the issue and in chat.
**Why:** a ruler change was made, merged and pushed with no issue; the owner: "vc tem que abrir a issue..
pq vc nao segue as mesmas regras do jovee rig/studio?".

## Screens use the jovee-theme controls

Before drawing or changing a control in `crates/jr-gui/ui/`, run `cargo update -p jovee-theme` and read
its `ui/` and `docs/patterns.md`. What it has (card, window header, segmented, caption, tag…) is imported,
never redrawn: a local variant is a bug. The ruler's `theme.slint` holds only what is the ruler's own.
**Why:** the theme gained shared controls while the ruler kept its own Panel, SegControl and Pill on an old
pin, so it looked different from the other JoVee programs until the owner pointed it out.

## Keep the repo and the docs current — always, without being asked

Any change to a `.scad`, an art generator, or `build.sh` is only finished when all of
this is done, in the same pass:

1. **Regenerate the artifacts it affects.** `./build.sh` rebuilds every STL and every
   PDF; for a single part, render just that STL. Committed STLs and PDFs must match the
   source that produced them.
2. **Update the docs that state the changed fact.** Part dimensions and weights live in
   both `README.md` and the piece's own `README.md`; fit numbers and tuning advice live
   in `scale/README.md`; how the piece is played lives in `docs/how-to-use.md`. A number
   that changed in the `.scad` and not in the tables is a bug.
3. **Commit and push to `origin/main`.** No need to ask first — pushing this repo is
   pre-authorized. Commit messages in Portuguese, everything else in English.

## Verify before claiming it works

- `build.sh` is the test suite: the art generators abort on any text that crosses a cut
  line, overflows a cell or runs off the page, and `scale/scale.scad` carries `assert()`s
  on the fit. A green build is the minimum, not the proof.
- **`assert()` only checks the numbers you thought of.** When a cut or a solid changes
  shape, probe the resulting solid — intersect the part with a thin column at chosen
  coordinates and read the material height back. A paper-recess cut once shipped with a
  wrong rotation, opening a slit through the side wall: every assert passed.
- OpenSCAD's STL output is **not** deterministic — the same source rendered twice gives
  different bytes. Never compare STL hashes to decide whether geometry changed; compare
  the source, or the bounding box, or probe it.

## The fit rules that were learned the hard way

These are enforced by `assert()` in `scale/scale.scad`. Do not relax one without
understanding which failure it prevents:

- **Dovetail grip ≥ 4 × side play.** A plate that can slide sideways further than its
  ledge is deep loses engagement on one side, lifts and falls out. Plate widths are
  derived from `FIT`; never type a width by hand.
- **The paper recess is never narrower than a plate that runs over it.** Otherwise the
  plate balances on two thin ledges, stays flat while the slider holds it down, and tips
  into the recess the moment it slides out from under it.
- **A lifted plate must not reach the channel above it.** Every channel is deliberately
  taller than its plate needs.
- **Ask what a part rests ON, across its whole width, in the worst position.** Twice now
  the same defect shipped: a plate balanced on ledges too narrow to hold it (a paper
  recess narrower than the plate, then channels stepped inwards until the ledge vanished).
  Both times the numbers that were asserted looked fine. Assert the bearing itself.
- **A long tail hanging off a short body must keep the dovetail profile**, or it pitches
  down into the channel below and jams. The blade's tails are rails, not strips.
- Changing `CW`, `RS` or `TUNING` means changing them in the `.scad` **and** in
  `scale/art_scale.py`. They have no shared source.

## Before asking for a 5-hour print

`scale_0-FIT-COUPON.stl` prints in ~10 minutes and tests the real cross-section. Any fit
change goes through the coupon first.

**The `.3mf` files in this repo are stale slicer projects, not build output.** `scale/scale.3mf`
(27/08), `scale/scale_0-FIT-COUPON.3mf` (28/08) and `scale/stl/1.3mf` (28/08) all predate the
bearing-ledge fix of 30/08, so they carry the geometry of the defect the plates fell through.
Nothing regenerates them. Always start a fresh slicer project from the STLs in `scale/stl/`;
opening a `.3mf` is how a 5-hour print gets thrown away.

- JoVee libraries (ruler, stem, theme, pedal drivers, tuner-core...) are declared with `branch = "main"`, never
  `rev`, `tag` or a version, and every build takes the tip of their `main`: `scripts/follow-branches.sh` moves
  Cargo.lock first (`scripts/update-libs.sh` before a build), and the commit carries
  the moved lock. **Why:** a Jam beta shipped a ruler four commits behind its `main`; the owner: "entre as
  dependencias do jovee nao pode ter trava por versao, sempre tem que ser a ultima versao da main".
