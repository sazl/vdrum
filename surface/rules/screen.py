"""The stationary screen (shown when no popup is up): title, name, value. The first rule whose
guards match is shown; the last is the default. Variables: {mode_slot} {mode_name} {kit}
{last_touched} {pattern} {bar} {next_pattern} {silent} {seq_track_number} {seq_track_name}."""


def screen(s):
    s.show(["{mode_slot} {mode_name}",
            "P{pattern} bar {bar}/4{next_pattern}{silent}",
            "Track {seq_track_number} {seq_track_name}"], when=["mode:sequencer"])
    s.show(["{mode_slot} {mode_name}", "{kit}", "{last_touched}"])
