"""Side-button LEDs (first matching rule wins, no match is off) and when the 16 buttons show the
mode slots. Colours are palette names (palette.py), section names, or `mode_pad`."""


def leds(l):
    l.mode_slots_when("shift")
    # Lit where they do something.
    l.button("logo", ([], "nav"))
    l.button("mute_select", ([], "nav"))
    # Sequencer view off (nav), on (white), sticky (orange).
    l.button("solo_arm", (["sticky"], "mode_active"), (["seq_view"], "white"), ([], "nav"))
    # Only in the sequencer view, where they can move.
    l.button("track_left", (["seq_view", "!seq_track_first"], "nav"))
    l.button("track_right", (["seq_view", "!seq_track_last"], "nav"))
    l.button("record", (["seq_view", "held:record"], "record_held"), (["seq_view"], "record_ready"))
    # Kits with Shift, bars in the sequencer view.
    l.button("page_up", (["shift"], "nav"), (["seq_view", "!seq_bar_first"], "nav"))
    l.button("page_down", (["shift"], "nav"), (["seq_view", "!seq_bar_last"], "nav"))
    # Red while silenced (host clock), green while running; otherwise start (mode 8, internal
    # clock) or the voice of a voice mode.
    l.button("play",
             (["seq_silent"], "play_silent"),
             (["seq_running"], "play_running"),
             (["mode:sequencer", "host_clock"], "off"),
             (["mode:sequencer"], "play_ready"),
             ([], "mode_pad"))
