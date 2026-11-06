"""Scenarios for navigation: modes, the sequencer view, the pattern layer, kit browsing."""

from surface_spec.scenario import Scenario


def scenarios():
    out = []

    s = Scenario("Shift + button N selects mode N; Back returns")
    s.expect({"surface.mode": "voice:0", "screen.lines": ["1 Drum 1", "Init", ""]})
    s.hold("SHIFT").expect({"leds.T1": "lit", "leds.T2": [40, 40, 40], "leds.B5": [0, 0, 0]})
    s.press("T8").release("SHIFT")
    s.expect({"surface.mode": "sequencer", "screen.lines": ["8 Sequencer", "P01 bar 1/4", "Track 1 D1"]})
    s.with_held("SHIFT", lambda s: s.press("B8"))
    s.expect({"surface.mode": "global", "screen.lines.0": "16 Global"})
    s.press("LOGO").expect({"surface.mode": "sequencer"})
    s.with_held("SHIFT", lambda s: s.press("B5"))
    s.expect({"surface.mode": "sequencer", "screen.source": "overlay", "screen.lines": ["Mode", "Slot 13", "empty"]})
    out.append(s)

    s = Scenario("Solo/Arm toggles the sequencer view; Shift + Solo/Arm makes it sticky")
    s.press("SOLO").expect({"surface.seq_view": True, "leds.SOLO": [254, 254, 254],
                            "screen.lines": ["Sequencer view", "on", "1 D1"], "cells.T1.label": "Step 1"})
    s.with_held("SHIFT", lambda s: s.press("T2"))
    s.expect({"surface.mode": "voice:1", "surface.seq_view": False, "leds.SOLO": [70, 76, 88]})
    s.with_held("SHIFT", lambda s: s.press("SOLO"))
    s.expect({"surface.sticky": True, "surface.seq_view": True, "leds.SOLO": [254, 80, 0]})
    s.with_held("SHIFT", lambda s: s.press("T3"))
    s.expect({"surface.mode": "voice:2", "surface.seq_view": True})
    out.append(s)

    s = Scenario("Track left/right select the sequencer track in the sequencer view only")
    s.press("TRACK_RIGHT").expect({"surface.seq_track": 0, "leds.TRACK_RIGHT": [0, 0, 0]})
    s.press("SOLO").expect({"leds.TRACK_LEFT": [0, 0, 0], "leds.TRACK_RIGHT": [70, 76, 88]})
    s.press("TRACK_RIGHT").press("TRACK_RIGHT")
    s.expect({"surface.seq_track": 2, "screen.lines": ["Sequencer", "Track", "3 D3"], "leds.TRACK_LEFT": [70, 76, 88]})
    s.press("PAGE_DOWN").expect({"surface.seq_bar": 1, "screen.lines": ["Sequencer", "Bar", "2 of 4"]})
    out.append(s)

    s = Scenario("Mute/Select: a short press goes to Perform; held in the sequencer view it is the pattern layer")
    s.press("MUTE").expect({"surface.mode": "perform", "screen.lines.0": "10 Perform"})
    s.with_held("SHIFT", lambda s: s.press("T8"))
    s.hold("MUTE").expect({"screen.lines": ["Sequencer", "Patterns", "patterns 1 to 16"], "leds.T1": [0, 254, 0]})
    s.press("T3").release("MUTE")
    s.expect({"surface.mode": "sequencer", "seq.pattern": 2})
    out.append(s)

    s = Scenario("Shift + Page browses kits; Play loads the kit shown")
    s.with_held("SHIFT", lambda s: s.press("PAGE_DOWN"))
    s.expect({"surface.browsing": True, "screen.lines": ["Kit", "Kit A", "Play loads"]})
    s.press("PLAY").expect({"kit": "Kit A", "surface.browsing": False, "screen.lines": ["Kit", "Kit A", "loaded"]})
    s.wait(1100).expect({"screen.source": "stationary", "screen.lines.1": "Kit A"})
    out.append(s)
    return out
