"""Scenarios for editing: encoders, faders with pickup, popups, steps."""

from surface_spec.scenario import Scenario


def scenarios():
    out = []

    s = Scenario("An encoder changes its parameter and shows a popup for the popup time")
    s.turn("K11", 2).expect({"screen.source": "overlay", "screen.lines": ["DRUM 1 Oscillator", "Fine", "2"],
                             "cells.K11.value": "2"})
    s.wait(900).expect({"screen.source": "overlay"})
    s.wait(200).expect({"screen.source": "stationary", "screen.lines": ["1 Drum 1", "Init", "Fine 2"]})
    out.append(s)

    s = Scenario("Faders pick up their value before they move it")
    s.with_held("SHIFT", lambda s: s.press("B1"))
    s.expect({"surface.mode": "mix_fx"})
    s.fader("F1", 30).expect({"cells.F1.fader.picked": False})
    s.fader("F1", 127).expect({"cells.F1.fader.picked": True})
    out.append(s)

    s = Scenario("Steps: a press sets a step; Record + step clears it")
    s.with_held("SHIFT", lambda s: s.press("T8"))
    s.press("T1").expect({"leds.T1": [126, 60, 44]})
    s.hold("REC").expect({"leds.REC": [254, 0, 0]}).press("T1").release("REC")
    s.expect({"leds.T1": [14, 6, 4], "leds.REC": [64, 0, 0]})
    out.append(s)

    s = Scenario("The sequencer runs on the internal clock")
    s.with_held("SHIFT", lambda s: s.press("B8"))
    s.turn("K38", 1).expect({"screen.lines~": "Internal"})
    s.with_held("SHIFT", lambda s: s.press("T8"))
    s.press("T1").press("T5").expect({"leds.PLAY": [0, 64, 0]})
    s.press("PLAY").expect({"seq.running": True})
    s.wait(10).expect({"leds.T1": [254, 254, 254], "leds.PLAY": [0, 254, 0]})
    s.wait(500).expect({"leds.T5": [254, 254, 254], "seq.step.0": 4})
    s.press("PLAY").expect({"seq.running": False})
    out.append(s)
    return out
