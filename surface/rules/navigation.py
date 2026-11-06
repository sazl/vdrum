"""What the side buttons and the 16 buttons do, by state. The first rule whose button and guards
match, and which has an entry for the event (press or release), handles it: `release=[]` handles a
release by doing nothing, a missing `release` lets it fall through to the next rule.

Guards: shift, seq_view, view_toggle, sticky, voice_mode, mode:<key>, held:<button>,
seq_track_first/last, seq_bar_first/last, seq_running, seq_silent, host_clock, browsing, step_held;
`!` negates. Steps: see surface_spec/rules.py (NAV_STEPS) and docs/lcxl3-daw-mode/surface-spec.md.
"""

from surface_spec import rules as r


def navigation(nav):
    nav.on("shift", press=[r.shift()], release=[r.shift()], note="Shift follows the button")
    nav.on("solo_arm", when=["shift"], press=[r.toggle_sticky()], release=[],
           note="Shift + Solo/Arm: the sequencer view stays on across mode changes")
    # Shift + button N selects the mode in slot N (top row 1..8, bottom row 9..16). Only the press:
    # the release falls through (to the step buttons in the sequencer view).
    nav.on("top", when=["shift"], press=[r.mode_slot(1)], note="Shift + top N: mode slot N")
    nav.on("bottom", when=["shift"], press=[r.mode_slot(9)], note="Shift + bottom N: mode slot 8 + N")
    nav.on("top", when=["seq_view"], press=[r.seq_button(0)], release=[r.seq_button(0)],
           note="Sequencer view: steps 1 to 8")
    nav.on("bottom", when=["seq_view"], press=[r.seq_button(8)], release=[r.seq_button(8)],
           note="Sequencer view: steps 9 to 16")
    # Track left/right select the sequencer track in the sequencer view; with Record held they
    # clear its pattern. Elsewhere they do nothing (modes are selected with Shift + button N).
    for button, d in (("track_left", -1), ("track_right", 1)):
        nav.on(button, when=["seq_view", "held:record"], press=[r.clear_seq_track()], release=[],
               note="Record + Track: clear the selected track")
        nav.on(button, when=["seq_view"], press=[r.seq_track_step(d)], release=[],
               note="Sequencer view: select the track")
        nav.on(button, press=[], release=[])
    nav.on("logo", press=[r.previous_mode()], release=[], note="Back to the previous mode")
    nav.on("solo_arm", when=["mode:sequencer"], press=[r.message("Sequencer view", "always on", "in mode 8")],
           release=[])
    nav.on("solo_arm", press=[r.toggle_seq_view()], release=[], note="Solo/Arm: sequencer view on/off")
    # In the sequencer view Mute/Select is the pattern layer while held; a short press that did not
    # use the layer still goes to Perform.
    nav.on("mute_select", when=["seq_view"],
           press=[r.layer_begin(), r.message("Sequencer", "Patterns", "patterns 1 to 16")],
           release=[r.mode_unless_layer_used("perform")], note="Pattern layer while held")
    nav.on("mute_select", press=[r.mode("perform")], release=[], note="Perform")
    for button, d in (("page_up", -1), ("page_down", 1)):
        nav.on(button, when=["shift"], press=[r.kit_browse(d)], release=[], note="Shift + Page: browse kits, Play loads")
        nav.on(button, when=["seq_view"], press=[r.seq_bar_step(d)], release=[], note="Sequencer view: bar")
        nav.on(button, press=[], release=[])
    # Record = Clear (2026-11-05): hold it and press a step to clear that step, or Track left/right
    # to clear the selected track's pattern.
    nav.on("record", press=[r.message("Clear", "hold +", "step / Track < >")], release=[])
    nav.on("play", press=[r.play()], release=[])
    nav.on("top", press=[r.cell()], release=[r.cell()], note="The mode's cell")
    nav.on("bottom", press=[r.cell()], release=[r.cell()], note="The mode's cell")
