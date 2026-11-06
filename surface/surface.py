"""The VDrum Launch Control XL 3 surface: which mode sits in which slot (Shift + button N), and the
overlays. To try a different mode, write a new function (in modes/ or a new file) and point a slot
at it here; to try a variant of a mode, copy its function, change it, and swap the line below.

Build with: python3 tools/surface_spec/run.py build
"""

from surface_spec import Surface

from components.common import steps
from modes.global_mode import global_mode
from modes.lfo_mod import lfo_mod
from modes.mix_fx import mix_fx
from modes.morph_macro import morph_macro
from modes.perform import perform
from modes.sequencer import sequencer, step_held
from modes.voices import closed_hat, cymbal, drum, open_hat, snare
from surface_spec import Layout

surface = Surface()
#             mode key       slot  name             layout id        layout
surface.mode("voice:0",      1,  "Drum 1",        "drum-1",        drum(0))
surface.mode("voice:1",      2,  "Drum 2",        "drum-2",        drum(1))
surface.mode("voice:2",      3,  "Drum 3",        "drum-3",        drum(2))
surface.mode("voice:3",      4,  "Snare",         "snare",         snare())
surface.mode("voice:4",      5,  "Cymbal",        "cymbal",        cymbal())
surface.mode("voice:5",      6,  "Closed hi-hat", "closed-hi-hat", closed_hat())
surface.mode("voice:6",      7,  "Open hi-hat",   "open-hi-hat",   open_hat())
surface.mode("sequencer",    8,  "Sequencer",     "sequencer",     sequencer())
surface.mode("mix_fx",       9,  "Mix + FX",      "mix-fx",        mix_fx())
surface.mode("perform",      10, "Perform",       "perform",       perform())
surface.mode("lfo_mod",      11, "LFO + Mod",     "lfo-mod",       lfo_mod())
surface.mode("morph_macro",  12, "Morph + Macro", "morph-macro",   morph_macro())
surface.mode("global",       16, "Global",        "global",        global_mode())

# Laid over any mode while the sequencer view (Solo/Arm) is on.
surface.overlay("steps", Layout().use(steps))
# Laid over the sequencer while a step button is held.
surface.overlay("step-held", step_held())
