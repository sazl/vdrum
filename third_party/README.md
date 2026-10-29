# Third-party code

* `lxr-firmware`: submodule, [erica-synths/lxr-desktop-app](https://github.com/erica-synths/lxr-desktop-app).
* `egui-baseview` (MIT): copy of [BillyDM/egui-baseview](https://github.com/BillyDM/egui-baseview)
  at `ec70c3fe6b2f070dcacbc22924431edbe24bd1c0`, the revision nih-plug pins. The wgpu renderer and
  the examples are removed. Change: egui's zoom factor is applied. The screen rectangle, pointer
  positions and pixel scroll deltas are divided by the zoom, the renderer uses egui's
  `pixels_per_point` (system scale × zoom), and `ViewportCommand::InnerSize` is converted from points
  to logical pixels. Upstream ignores the zoom, so `Context::set_zoom_factor` had no visible effect.
* `nih_plug_egui` (ISC): copy of the crate in
  [robbert-vdh/nih-plug](https://github.com/robbert-vdh/nih-plug) at
  `de421011f41a6d10fc8c7a6084e4f4dee0143683`, using the copy above. Changes:
  * `EguiState::set_requested_size` is public, and `requested_size` and `set_size` are added, so
    the editor can resize its window (zoom) and migrate saved sizes.
  * A requested size is stored before the host is asked to resize. The CLAP and standalone
    wrappers read `Editor::size()` during the request; upstream had already cleared the request
    and not yet stored the new size, so they were given the old size.
  * A resize sets the physical size from the logical size times the system scale (upstream used
    the logical size, which is half the physical size on a Retina display) and converts the
    viewport command to points using the zoom.
  * The corner resize converts points to logical pixels using the zoom.
