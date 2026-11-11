#!/usr/bin/env python3
"""Actual source identities and malformed-input tests for the audio importer."""
import copy
import importlib.util
import json
from pathlib import Path
import struct
import tempfile
import unittest

import naraka_audio as audio

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "assets/prepared/naraka/audio"
spec = importlib.util.spec_from_file_location("naraka_audio_importer", Path(__file__).with_name("import-naraka-audio.py"))
importer = importlib.util.module_from_spec(spec)
spec.loader.exec_module(importer)


class OriginalAudioContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.manifest, cls.media = importer.load_prepared(PACK)

    def bank(self, name):
        path = self.manifest["banks"][name]
        bank = json.loads((PACK / path).read_text())
        bank["objects"] = {obj["id"]: obj for obj in bank["objects"]}
        return bank

    def test_original_three_stages_keep_distinct_events_and_all_variants(self):
        identities = []
        for index, expected in [(1, 2), (2, 3), (3, 3)]:
            name = f"gman_hengdao_attack_light_0{index}"
            event = self.manifest["events"][name]
            self.assertEqual(len(event["layers"]), 1)
            sounds = event["layers"][0]["sounds"]
            self.assertEqual(len(sounds), expected)
            self.assertAlmostEqual(sum(s["weight"] for s in sounds), 1)
            identities.append({s["source_id"] for s in sounds})
        self.assertFalse(identities[0] & identities[1] | identities[1] & identities[2])

    def test_original_landing_keeps_both_actions_and_actual_delay(self):
        layers = self.manifest["events"]["move_jump_land"]["layers"]
        self.assertEqual([l["original_delay_ms"] for l in layers], [100, 0])
        self.assertEqual([l["delay"] for l in layers], [0.1, 0])
        self.assertNotEqual(layers[0]["action_id"], layers[1]["action_id"])
        self.assertEqual([len(l["sounds"]) for l in layers], [8, 8])
        # They are two original actions over a shared eight-media set, not 16
        # independent WEMs and not one layer with 16 random choices.
        self.assertEqual({s["source_id"] for s in layers[0]["sounds"]},
                         {s["source_id"] for s in layers[1]["sounds"]})

    def test_original_gun_and_bow_firing_keep_three_variants_and_single_action(self):
        for name in ["gman_fk_attack_01", "gman_bow_attack_01"]:
            event = self.manifest["events"][name]
            self.assertEqual(len(event["layers"]), 1)
            layer = event["layers"][0]
            self.assertEqual(layer["delay"], 0)
            self.assertEqual(len(layer["sounds"]), 3)
            self.assertAlmostEqual(sum(s["weight"] for s in layer["sounds"]), 1)

    def test_ranged_source_tracks_distinguish_actual_fire_and_empty_sniper(self):
        evidence = json.loads((PACK / "source-evidence/audio-tracks-selected.json").read_text())
        tracks = {r["key"]: r for r in evidence["selected"]}

        def event_names(name):
            return [s["sound"]["event"] for g in tracks[name]["groups"]
                    for e in g["sound_events"] for s in e["sounds"]]

        self.assertIn("gman_bow_attack_01", event_names("male_bow_attack_01"))
        self.assertIn("gman_fk_attack_01", event_names("male_fk_attack_01"))
        self.assertIn("gman_fk_attack_01_barrett", event_names("male_fk_attack_01"))
        self.assertEqual(event_names("male_sniper_attack_01"), [])
        self.assertEqual(event_names("male_sniper_ads_attack_01"), [])

    def test_unknown_action_property_is_rejected_instead_of_silently_dropped(self):
        bank = self.bank("move_jump")
        action_id = self.manifest["events"]["move_jump_land"]["layers"][0]["action_id"]
        action = copy.deepcopy(bank["objects"][action_id])
        body = bytearray.fromhex(action["body_hex"])
        self.assertEqual((body[7], body[8]), (1, 0x0F))
        body[8] = 0x0E
        action["body_hex"] = body.hex()
        with self.assertRaisesRegex(ValueError, "unsupported PlayAction property"):
            audio.play_action(action, bank["bank_id"])

    def test_real_random_container_cycle_is_rejected(self):
        bank = self.bank("gman_hengdao")
        target = self.manifest["events"]["gman_hengdao_attack_light_01"]["layers"][0]["target"]
        node = bank["objects"][target]
        tail = audio.random_tail(node, {o["id"] for o in bank["object_index"]})
        body = bytearray.fromhex(node["body_hex"])
        old = tail["children"][0]
        struct.pack_into("<I", body, tail["offset"] + 4, target)
        playlist_at = tail["offset"] + 4 + len(tail["children"]) * 4 + 2
        for i, row in enumerate(tail["playlist"]):
            if row["id"] == old:
                struct.pack_into("<I", body, playlist_at + i * 8, target)
        node["body_hex"] = body.hex()
        with self.assertRaisesRegex(ValueError, "cycle"):
            audio.event_layers(bank["objects"], bank["object_index"],
                               bank["bank_id"], "gman_hengdao_attack_light_01")

    def test_source_wem_length_corruption_is_rejected(self):
        row = next(iter(self.media.values()))
        data = bytearray((PACK / row["wem"]).read_bytes())
        data[4] ^= 1
        with self.assertRaisesRegex(ValueError, "RIFF size mismatch"):
            audio.check_wem(data)

    def test_frozen_pcm_corruption_is_rejected(self):
        row = next(iter(self.media.values()))
        metadata = dict(row["decoded"], sha256="0" * 64)
        with self.assertRaisesRegex(ValueError, "PCM SHA/length mismatch"):
            audio.unpack_pcm(PACK / row["pcm"], metadata)


class ArtifactBoundaries(unittest.TestCase):
    def test_pck_alignment_and_extent_are_checked(self):
        with tempfile.TemporaryDirectory(prefix="naraka-pck-test-") as temporary:
            path = Path(temporary) / "fixture.pck"
            bank = struct.pack("<I5I", 1, 7, 1, 8, 64, 0)
            data = struct.pack("<4s6I", b"AKPK", 52, 1, 0, 24, 4, 4) + bank + bytes(8)
            path.write_bytes(data + bytes(72 - len(data)))
            self.assertEqual(audio.read_pck(path)["entries"][0]["offset"], 64)
            data = bytearray(path.read_bytes())
            struct.pack_into("<I", data, 36, 0)
            path.write_bytes(data)
            with self.assertRaisesRegex(ValueError, "alignment"):
                audio.read_pck(path)

    def test_portable_manifest_rejects_escape_symlink_and_corruption(self):
        with tempfile.TemporaryDirectory(prefix="naraka-path-test-") as temporary:
            root = Path(temporary)
            (root / "source").write_bytes(b"source")
            (root / "prepared-manifest.json").write_bytes(b"{}")
            row = dict(path="source", bytes=6, sha256=audio.sha(b"source"))
            audio.check_file_list(root, {"files": [row]})
            with self.assertRaisesRegex(ValueError, "relative"):
                audio.safe_file(root, "../source")
            (root / "source").write_bytes(b"edited")
            with self.assertRaisesRegex(ValueError, "changed"):
                audio.check_file_list(root, {"files": [row]})
            (root / "alias").symlink_to(root / "source")
            with self.assertRaisesRegex(ValueError, "symlink"):
                audio.safe_file(root, "alias")


if __name__ == "__main__":
    unittest.main()
