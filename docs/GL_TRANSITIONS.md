# GL Transitions

Enable **Settings → Content libraries → GL Transitions**, then open the
**Transitions → GL Transitions** category. Search and favourites use the normal
transition library controls. Hover a card to download its shader and preview it;
apply it to the selected cut or drag it to a cut on the timeline.

The official collection is pinned to **1.76.1** (125 entries). Clovercut supports
123 two-image transitions. `displacement` and `luma` need an additional texture
input and are excluded. The six existing Clovercut GL packages retain their IDs
and take the place of the corresponding online entries, avoiding duplicate rows.

The first catalogue request needs internet. Metadata is stored under the app's
data directory in `gl-transitions/`; shaders are downloaded individually only
when previewed or used. Installed packages live in the existing configuration
`effects/gltransitions.<name>/` directory and work offline on subsequent starts.
Turning the library off hides its entries and stops new requests; it does not
remove installed packages or break transitions already used by a project.

Every downloaded WGSL shader is adapted to Clovercut's two-image shader contract
and parsed/validated before an atomic package install. The monitor, hover preview
and export use that same installed package. Original vector parameters are exposed
as scalar component controls, with their original defaults. Original sources,
author/license metadata and the collection's LICENSE are retained with the package.

Downloads do not block the UI. Leaving a card cancels its preview, and changing
selection/project while an application download is running prevents it from
being applied to the wrong cut. Failed catalogue/application requests show an
error; the six built-in GL transitions remain available offline at all times.

## Validation

The opt-in `concat-host` integration test `gl_transitions_live` checks a real
catalogue download, shader installation, persistent cache reuse and GPU rendering
at the start/middle/end of all 123 supported transitions. GPU fixture verification
uses `CLOVERCUT_GL_INDEX` pointing to the official `gl-transitions.json` and
`CLOVERCUT_GL_FIXTURES` pointing to its `wgsl/` directory; it is not run by ordinary
CI, so normal tests do not require network access or a hardware GPU.
