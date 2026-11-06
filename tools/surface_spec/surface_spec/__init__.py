"""VDrum surface spec: author the Launch Control XL 3 modes in Python, build them into
crates/vdrum-surface/spec/surface.json. See docs/lcxl3-daw-mode/surface-spec.md."""

from .behaviours import *  # noqa: F401,F403
from .dsl import *  # noqa: F401,F403
from .voice import vid  # noqa: F401
