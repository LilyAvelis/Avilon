"""Synthetic behavioral checks; never opens a user profile or real images."""
import os
import sqlite3
import tempfile
import unittest
from pathlib import Path
from state_contract import SCHEMA, revision, reorder, page
from image_contract import Cache

class StorageContract(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.path = str(Path(self.temp.name) / "synthetic.sqlite")
        self.db = sqlite3.connect(self.path)
        self.addCleanup(self.db.close)
        self.db.executescript(SCHEMA + "CREATE TABLE items(scope TEXT NOT NULL,id INTEGER NOT NULL,PRIMARY KEY(scope,id));")
        for scope in ("alpha", "beta"):
            self.db.executemany("INSERT INTO slots VALUES(?,?,?)",[(scope,1,"same"),(scope,2,"same"),(scope,3,"blue")])
            self.db.execute("INSERT INTO revisions VALUES(?,0)",(scope,))
        self.db.executemany("INSERT INTO items VALUES('alpha',?)",[(1,),(2,),(3,),(9,)])
        self.db.executemany("INSERT INTO assignments VALUES(?,?,?)",[("alpha",1,1),("alpha",2,2),("beta",1,1)])
        self.db.commit()

    def test_reorder_moves_identity_and_assignments_isolates_scope_and_aba(self):
        initial = revision(self.db,"alpha")
        reorder(self.db,"alpha",[2,3,1],initial)
        self.assertEqual(self.db.execute("SELECT slot FROM assignments WHERE scope='alpha' AND item_id=1").fetchone()[0],3)
        self.assertEqual(self.db.execute("SELECT slot FROM assignments WHERE scope='beta' AND item_id=1").fetchone()[0],1)
        reorder(self.db,"alpha",[3,1,2],revision(self.db,"alpha"))
        self.assertNotEqual(revision(self.db,"alpha"),initial)
        self.assertEqual(self.db.execute("SELECT number,label FROM slots WHERE scope='alpha' ORDER BY number").fetchall(),[(1,"same"),(2,"same"),(3,"blue")])
        with self.assertRaisesRegex(ValueError,"stale revision"):
            reorder(self.db,"alpha",[2,1,3],initial)
        self.assertEqual(self.db.execute("SELECT slot FROM assignments WHERE scope='alpha' AND item_id=1").fetchone()[0],1)

    def test_reorder_failed_write_rolls_back_definitions_and_revision(self):
        original = revision(self.db,"alpha")
        self.db.execute("CREATE TRIGGER fail_assignment BEFORE UPDATE ON assignments BEGIN SELECT RAISE(ABORT,'synthetic failure'); END")
        self.db.commit()
        with self.assertRaises(sqlite3.IntegrityError):
            reorder(self.db,"alpha",[2,3,1],original)
        self.assertEqual(revision(self.db,"alpha"),original)
        self.assertEqual(self.db.execute("SELECT label FROM slots WHERE scope='alpha' AND number=3").fetchone()[0],"blue")
        self.assertEqual(self.db.execute("SELECT slot FROM assignments WHERE scope='alpha' AND item_id=1").fetchone()[0],1)

    def test_global_tag_grouping_precedes_limit_and_second_writer_invalidates(self):
        first,cursor = page(self.db,"alpha",1)
        self.assertEqual(first,[(1,1)])  # untagged newest id 9 must not win the first page
        second,cursor2 = page(self.db,"alpha",2,cursor)
        self.assertEqual(second,[(2,2),(9,999)])
        last,cursor3 = page(self.db,"alpha",2,cursor2)
        self.assertEqual(last,[(3,999)])
        self.assertIsNone(cursor3)
        other = sqlite3.connect(self.path)
        try:
            other.execute("UPDATE assignments SET slot=3 WHERE scope='alpha' AND item_id=1")
            other.commit()
            with self.assertRaisesRegex(ValueError,"stale cursor"):
                page(self.db,"alpha",2,cursor)
        finally:
            other.close()

class ImageContract(unittest.TestCase):
    def test_carry_only_decoded_images_discard_late_work_and_revisions(self):
        cache=Cache();cache.change_context(("images",None,False),1)
        image=cache.request(1,320,"content-a");self.assertTrue(cache.complete(image,"image","texture-a"))
        shell=cache.request(2,320,"file-a");self.assertTrue(cache.complete(shell,"shell_icon","shell"))
        empty=cache.request(4,320,"empty");self.assertTrue(cache.complete(empty,"image",None))
        old_job=cache.request(3,320,"pending-a")
        cache.change_context(("images",None,False),2)
        self.assertEqual(cache.resident[(1,320,"content-a")],("image","texture-a"))
        self.assertNotIn((2,320,"file-a"),cache.resident)
        self.assertNotIn((4,320,"empty"),cache.resident)
        self.assertFalse(cache.complete(old_job,"image","stale"))
        new_revision=cache.request(1,320,"content-b")
        self.assertIn(new_revision,cache.pending)
        cache.change_context(("folder","other",False),3)
        self.assertFalse(cache.resident)
        self.assertFalse(cache.complete(new_revision,"image","wrong-folder"))

if __name__ == "__main__":
    unittest.main()
